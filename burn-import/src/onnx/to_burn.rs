use std::{
    collections::HashSet,
    env,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

use burn::{
    nn::conv::Conv2dPaddingConfig,
    record::{DefaultFileRecorder, FullPrecisionSettings, PrettyJsonFileRecorder, Recorder},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Type};

use crate::onnx::{
    ir::{ArgType, Node, NodeType},
    op_configuration::{
        batch_norm_config, conv2d_config, flatten_config, linear_config, log_softmax_config,
    },
    shape_inference::first_input_dim,
};

use super::{from_onnx::parse_onnx, ir::Graph};

use rust_format::{Config, Edition, Formatter, PostProcess, RustFmt};

/// Generate code and states from `.onnx` files and save them to the `out_dir`.
#[derive(Debug, Default)]
pub struct ModelGen {
    out_dir: Option<PathBuf>,
    /// List of onnx files to generate source code from.
    inputs: Vec<PathBuf>,
    development: bool,
}

impl ModelGen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output directory.
    pub fn out_dir(&mut self, out_dir: &str) -> &mut Self {
        self.out_dir = Some(Path::new(out_dir).into());
        self
    }

    /// Add input file.
    pub fn input(&mut self, input: &str) -> &mut Self {
        self.inputs.push(input.into());
        self
    }

    /// Set development mode.
    ///
    /// If this is set to true, the generated model will be saved as `.graph.txt` files and model
    /// states will be saved as `.json` file.
    pub fn development(&mut self, development: bool) -> &mut Self {
        self.development = development;
        self
    }

    /// Run code generation.
    ///
    /// This function is intended to be called from `build.rs` script.
    pub fn run_from_script(&self) {
        self.run(true);
    }

    /// Run code generation.
    ///
    /// This function is intended to be called from CLI.
    pub fn run_from_cli(&self) {
        self.run(false);
    }

    pub fn code_formatter() -> RustFmt {
        let config = Config::new_str()
            .post_proc(PostProcess::ReplaceMarkersAndDocBlocks)
            .edition(Edition::Rust2021);

        RustFmt::from_config(config)
    }

    /// Run code generation.
    fn run(&self, is_build_script: bool) {
        // prepend the out_dir to the cargo_out_dir if this is a build script
        let out_dir = if is_build_script {
            let cargo_out_dir = env::var("OUT_DIR").expect("OUT_DIR env is not set");
            let mut path = PathBuf::from(cargo_out_dir);

            // // Append the out_dir to the cargo_out_dir
            path.push(self.out_dir.clone().unwrap());
            path
        } else {
            self.out_dir.as_ref().expect("out_dir is not set").clone()
        };

        let rust_formatter = Self::code_formatter();

        create_dir_all(&out_dir).unwrap();

        for input in self.inputs.iter() {
            let file_name = input.file_stem().unwrap();
            let out_file: PathBuf = out_dir.join(file_name);

            Self::generate_model(self.development, input, out_file, &rust_formatter);
        }
    }

    /// Generate model source code and model state.
    fn generate_model(
        development: bool,
        input: &PathBuf,
        out_file: PathBuf,
        rust_formatter: &RustFmt,
    ) {
        if development {
            let graph = parse_onnx(input.as_ref());
            // export the graph
            let debug_graph = format!("{:#?}", graph);
            fs::write(out_file.with_extension("graph.txt"), debug_graph).unwrap();
        }

        // export the source code
        let model = ModelSourceCode::new(input, &out_file);
        let code_str = rust_formatter.format_tokens(model.body()).unwrap();
        fs::write(out_file.with_extension("rs"), code_str).unwrap();

        // export the model state
        if development {
            let recorder = PrettyJsonFileRecorder::<FullPrecisionSettings>::new();
            recorder.record(model.graph, out_file).unwrap();
        } else {
            let recorder = DefaultFileRecorder::<FullPrecisionSettings>::new();
            recorder.record(model.graph, out_file).unwrap();
        }
    }
}

/// A model that can be used to generate code
#[derive(Debug, Clone)]
pub struct ModelSourceCode {
    onnx_path: PathBuf,
    record_path: PathBuf,
    pub graph: Graph,
}

impl ModelSourceCode {
    /// Create a new model from the onnx file
    pub fn new<P: AsRef<Path>>(onnx_path: P, record_path: P) -> Self {
        let graph = parse_onnx(onnx_path.as_ref());
        Self {
            onnx_path: onnx_path.as_ref().to_path_buf(),
            record_path: record_path.as_ref().to_path_buf(),
            graph,
        }
    }

    /// Generates source code for the model
    pub fn body(&self) -> TokenStream {
        let input = "Model"; // TODO make this a parameter
        let input = Ident::new(input, Span::call_site());

        let declaration = self.declaration(&input);

        let file_path = self.onnx_path.to_str().unwrap();

        let top_file_comment = format!("Generated from {file_path} by burn-import");

        let mut imports: HashSet<String> = HashSet::new();

        let implementation = self.implementation(&mut imports);

        let import_statements = self.import_statements(&imports);

        let shape_constants = self.shape_constants();

        //TODO print out the old -> new name mapping
        quote! {
            _comment_!(#top_file_comment);
            _blank_!();
            _blank_!();
            #import_statements
            _blank_!();
            #shape_constants
            _blank_!();
            #declaration
            _blank_!();
            #[allow(dead_code)]
            #[allow(clippy::new_without_default)]
            #[allow(clippy::let_and_return)]
            #implementation

        }
    }

    fn shape_constants(&self) -> TokenStream {
        let input_constants = self.graph.inputs.iter().enumerate().map(|(i, input)| {
            let name = format!("INPUT{}_SHAPE", i + 1);
            let name = Ident::new(&name, Span::call_site());
            let ArgType::Tensor(tensor) = input.clone().arg_type.unwrap();
            let dims = tensor.shape;
            let dims_count = dims.len();
            quote! {
                pub const #name: [usize; #dims_count] = [#(#dims),*];
            }
        });

        let output_constants = self.graph.outputs.iter().enumerate().map(|(i, input)| {
            let name = format!("OUTPUT{}_SHAPE", i + 1);
            let name = Ident::new(&name, Span::call_site());
            let ArgType::Tensor(tensor) = input.clone().arg_type.unwrap();
            let dims = tensor.shape;
            let dims_count = dims.len();
            quote! {
                pub const #name: [usize; #dims_count] = [#(#dims),*];
            }
        });

        quote! {
            #(#input_constants)*
            #(#output_constants)*
        }
    }

    /// Generates import statements for the model
    fn import_statements(&self, imports: &HashSet<String>) -> TokenStream {
        let mut import_tokens = vec![];

        for import in imports.iter() {
            let path: syn::Path =
                syn::parse_str(import).expect("Unable to parse input string as a path");

            import_tokens.push(quote! { #path });
        }

        quote! {
            use burn::{
                module::Module,
                nn,
                tensor::{backend::Backend, Tensor},
                record::{Recorder, DefaultRecorder}
            };

            #(use #import_tokens;)*
        }
    }

    /// Generates the declaration portion of the source code for the model
    fn declaration(&self, name: &Ident) -> TokenStream {
        let fields = self.declaration_fields();

        let mut field_names = vec![];
        let mut field_types = vec![];

        for (field_name, field_type) in fields.iter() {
            field_names.push(field_name);
            field_types.push(field_type);
        }

        quote! {
            // TODO add documentation
            #[doc = "This is a generated model from an ONNX file"]
            #[derive(Module, Debug)]
            pub struct #name<B: Backend> {
                #(
                   #field_names: #field_types,
                )*
            }

        }
    }

    /// Model implementation code
    fn implementation(&self, imports: &mut HashSet<String>) -> TokenStream {
        let forward_method = self.forward_method(imports);

        let new_method = self.new_method();
        let load_state = self.load_state();

        quote! {
            impl<B: Backend> Model<B> {
                #new_method
                #forward_method
                #load_state
            }
        }
    }

    fn load_state(&self) -> TokenStream {
        let file_path = self.record_path.to_str().unwrap().replace('\\', "\\\\");
        quote! {
            pub fn load_state(self) -> Self {
                let recorder = DefaultRecorder::new();
                let record = recorder.load(#file_path.into()).unwrap();

                self.load_record(record)
            }
        }
    }

    /// Generates the new method for the model
    fn forward_method(&self, imports: &mut HashSet<String>) -> TokenStream {
        let inputs = self.forward_signature_input();
        let return_type = self.forward_signature_return();
        let results = self.forward_method_results();

        let mut call_nodes: Vec<TokenStream> = vec![];

        for node in self.graph.nodes.iter() {
            if node.is_stateful {
                call_nodes.push(Self::node_call_stateful(node));
            } else {
                call_nodes.push(Self::node_call_stateless(node, imports));
            }
        }

        quote! {
            pub fn forward(&self, #(#inputs,)*) -> #return_type {
                #(#call_nodes)*
                #results
            }
        }
    }

    /// Generates source code for the stateful node calls, i.e. conv, dropout, etc.
    fn node_call_stateful(node: &Node) -> TokenStream {
        if !node.is_stateful {
            panic!("Node must be stateful");
        }

        let name = Ident::new(&node.name, Span::call_site());

        let mut inputs = vec![];

        for input in node.inputs.iter() {
            let name = Ident::new(&input.name, Span::call_site());
            inputs.push(quote! {
                #name
            });
        }

        let mut outputs = vec![];

        for output in node.outputs.iter() {
            let name = Ident::new(&output.name, Span::call_site());
            outputs.push(quote! {
                #name
            });
        }

        if outputs.len() == 1 {
            let output = outputs.pop().unwrap();
            quote! {
                let #output = self.#name.forward(#(#inputs,)*);
            }
        } else {
            quote! {
                let (#(#outputs,)*) = self.#name.forward(#(#inputs,)*);
            }
        }
    }

    /// Generates source code for the forward method results
    fn forward_method_results(&self) -> TokenStream {
        let mut outputs = vec![];
        for output in self.graph.outputs.iter() {
            let name = Ident::new(&output.name, Span::call_site());
            outputs.push(quote! {
                #name
            });
        }
        if outputs.len() == 1 {
            let output = outputs.pop().unwrap();
            quote! {
                #output
            }
        } else {
            quote! {
                (#(#outputs,)*)
            }
        }
    }

    /// Generates source code for the stateless node calls, i.e. add, mul, etc.
    fn node_call_stateless(node: &Node, imports: &mut HashSet<String>) -> TokenStream {
        if node.is_stateful {
            panic!("Node must be stateless");
        }

        let mut inputs = vec![];

        for input in node.inputs.iter() {
            let name = Ident::new(&input.name, Span::call_site());
            inputs.push(quote! {
                #name
            });
        }

        let mut outputs = vec![];

        for output in node.outputs.iter() {
            let name = Ident::new(&output.name, Span::call_site());
            outputs.push(quote! {
                #name
            });
        }

        let rhs = Self::node_call_stateless_rhs(node, imports);

        if outputs.len() == 1 {
            let output = outputs.pop().unwrap();
            quote! {
                let #output = #rhs;
            }
        } else {
            quote! {
                let (#(#outputs,)*) = #rhs;
            }
        }
    }

    /// Generates source code for the right hand side stateless node calls, i.e. add, relu, etc.
    fn node_call_stateless_rhs(node: &Node, imports: &mut HashSet<String>) -> TokenStream {
        let mut inputs = vec![];

        for input in node.inputs.iter() {
            let name = Ident::new(&input.name, Span::call_site());
            inputs.push(quote! {
                #name
            });
        }

        let input1 = inputs.pop().unwrap();

        match node.node_type {
            NodeType::Relu => {
                imports.insert("burn::tensor::activation::relu".to_string());

                quote! { relu(#input1) }
            }
            NodeType::LogSoftmax => {
                imports.insert("burn::tensor::activation::log_softmax".to_string());
                let dim = log_softmax_config(node);

                quote! { log_softmax(#input1, #dim) }
            }
            NodeType::Flatten => {
                let (start_dim, end_dim) = flatten_config(node);

                quote! { #input1.flatten(#start_dim, #end_dim) }
            }
            _ => quote! {},
        }
    }

    /// Generates the forward method signature
    fn forward_signature_input(&self) -> Vec<TokenStream> {
        let mut fields = vec![];

        for input in self.graph.inputs.iter() {
            let name = Ident::new(&input.name, Span::call_site());

            let ty = match input.arg_type.as_ref().unwrap() {
                ArgType::Tensor(tensor) => {
                    let d = &tensor.shape.len();
                    syn::parse_str::<Type>(format!("Tensor<B, {d}>").as_str()).unwrap()
                }
            };

            fields.push(quote! {
                #name: #ty
            });
        }
        fields
    }

    /// Generates the forward method return signature
    fn forward_signature_return(&self) -> TokenStream {
        let mut field_types = vec![];

        for output in self.graph.outputs.iter() {
            let ty = match output.arg_type.as_ref().unwrap() {
                ArgType::Tensor(tensor) => {
                    let d = &tensor.shape.len();
                    syn::parse_str::<Type>(format!("Tensor<B, {d}>").as_str()).unwrap()
                }
            };

            field_types.push(ty);
        }

        if field_types.len() == 1 {
            // Return one output
            quote! {
                #(
                    #field_types
                 )*
            }
        } else {
            // Return a tuple of the outputs
            quote! {
                (#(
                    #field_types,
                 )*)
            }
        }
    }

    /// Generates source code for the initialization method
    fn new_method(&self) -> TokenStream {
        let initialization_fields = self.initialization_fields();

        let field_names = self.graph.nodes.iter().filter(|x| x.is_stateful).map(|x| {
            let name = Ident::new(&x.name, Span::call_site());
            quote! {
                #name
            }
        });

        quote! {
            pub fn new() -> Self {
                #(
                    #initialization_fields
                )*

                Self {
                    #(
                        #field_names
                    ),*
                }
            }
        }
    }

    /// Get the fields for the declaration of the model
    fn declaration_fields(&self) -> Vec<(Ident, Type)> {
        let mut fields = vec![];

        for node in self.graph.nodes.iter().filter(|x| x.is_stateful) {
            let node_type = match node.node_type {
                NodeType::Conv1d => syn::parse_str::<Type>("nn::conv::Conv1d<B>").unwrap(),
                NodeType::Conv2d => syn::parse_str::<Type>("nn::conv::Conv2d<B>").unwrap(),
                NodeType::Linear => syn::parse_str::<Type>("nn::Linear<B>").unwrap(),

                NodeType::BatchNormalization => batch_norm_type(node),
                _ => {
                    todo!("Node type not implemented: {:?}", node.node_type)
                }
            };

            let node_name = Ident::new(&node.name, Span::call_site());

            fields.push((node_name, node_type));
        }

        fields
    }

    /// Generates source code for the initialization method
    fn initialization_fields(&self) -> Vec<TokenStream> {
        let mut fields = vec![];

        for node in self.graph.nodes.iter().filter(|x| x.is_stateful) {
            let init_code = match node.node_type {
                NodeType::Conv2d => conv2d_init(node),
                NodeType::Linear => linear_init(node),
                NodeType::BatchNormalization => batch_norm_init(node),
                _ => {
                    todo!("Node type not implemented: {:?}", node.node_type)
                }
            };

            fields.push(init_code);
        }

        fields
    }
}

/// Generates source code for the initialization of a Conv2d node
fn conv2d_init(node: &Node) -> TokenStream {
    let node_name = Ident::new(&node.name, Span::call_site());

    let config = conv2d_config(node);

    let channel_in = config.channels[0];
    let channel_out = config.channels[1];
    let kernel_size_0 = config.kernel_size[0];
    let kernel_size_1 = config.kernel_size[1];
    let bias = config.bias;

    let padding = match config.padding {
        Conv2dPaddingConfig::Valid => quote! { nn::conv::Conv2dPaddingConfig::Valid },
        Conv2dPaddingConfig::Same => quote! { nn::conv::Conv2dPaddingConfig::Same },
        _ => todo!("Padding ({:?}) not implemented", config.padding),
    };

    quote! {
        let #node_name = nn::conv::Conv2dConfig::new([#channel_in, #channel_out], [#kernel_size_0, #kernel_size_1])
            .with_padding(#padding)
            .with_bias(#bias)
            .init();

    }
}

/// Generates source code for the initialization of a Linear node
fn linear_init(node: &Node) -> TokenStream {
    let node_name = Ident::new(&node.name, Span::call_site());
    let config = linear_config(node);

    let bias = config.bias;
    let input_size = config.d_input;
    let output_size = config.d_output;

    quote! {
        let #node_name = nn::LinearConfig::new(#input_size, #output_size)
            .with_bias(#bias)
            .init();

    }
}

/// Generates source code for the initialization of a BatchNorm node
fn batch_norm_init(node: &Node) -> TokenStream {
    let node_name = Ident::new(&node.name, Span::call_site());
    let config = batch_norm_config(node);

    let num_features = config.num_features;
    let epsilon = config.epsilon;
    let momentum = config.momentum;

    quote! {
        let #node_name = nn::BatchNormConfig::new(#num_features)
            .with_epsilon(#epsilon)
            .with_momentum(#momentum)
            .init();
    }
}

/// Figure out the BatchNorm type.
///
/// We need to figure out the dimensionality of the input to BatchNorm
fn batch_norm_type(node: &Node) -> Type {
    // Infer the dimensionality of BatchNorm the input (if 4, then 2D, if 5, then 3D)
    let dim = first_input_dim(node).unwrap() - 2;

    let ty = format!("nn::BatchNorm<B,{}>", dim);

    syn::parse_str::<Type>(ty.as_str()).unwrap()
}
