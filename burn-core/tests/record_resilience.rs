#[cfg(feature = "std")]
mod tests {
    use burn::{
        module::Module,
        nn,
        record::{
            BinFileRecorder, DefaultFileRecorder, FileRecorder, FullPrecisionSettings,
            PrettyJsonFileRecorder, RecorderError,
        },
    };
    use burn_core as burn;
    use burn_tensor::backend::Backend;
    use std::path::PathBuf;

    type TestBackend = burn_ndarray::NdArrayBackend<f32>;

    #[derive(Module, Debug)]
    pub struct Model<B: Backend> {
        linear1: nn::Linear<B>,
        linear2: nn::Linear<B>,
    }

    #[derive(Module, Debug)]
    pub struct ModelNewOptionalField<B: Backend> {
        linear1: nn::Linear<B>,
        linear2: nn::Linear<B>,
        new_field: Option<usize>,
    }

    #[derive(Module, Debug)]
    pub struct ModelNewFieldOrders<B: Backend> {
        linear2: nn::Linear<B>,
        linear1: nn::Linear<B>,
    }

    #[test]
    fn deserialize_with_new_optional_field_works_with_default_file_recorder() {
        deserialize_with_new_optional_field(
            "default",
            DefaultFileRecorder::<FullPrecisionSettings>::new(),
        )
        .unwrap();
    }

    #[test]
    fn deserialize_with_new_field_order_works_with_default_file_recorder() {
        deserialize_with_new_field_order(
            "default",
            DefaultFileRecorder::<FullPrecisionSettings>::new(),
        )
        .unwrap();
    }
    #[test]
    fn deserialize_with_new_optional_field_works_with_pretty_json() {
        deserialize_with_new_optional_field(
            "pretty-json",
            PrettyJsonFileRecorder::<FullPrecisionSettings>::new(),
        )
        .unwrap();
    }

    #[test]
    fn deserialize_with_new_field_order_works_with_pretty_json() {
        deserialize_with_new_field_order(
            "pretty-json",
            PrettyJsonFileRecorder::<FullPrecisionSettings>::new(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn deserialize_with_new_optional_field_doesnt_works_with_bin_file_recorder() {
        deserialize_with_new_optional_field("bin", BinFileRecorder::<FullPrecisionSettings>::new())
            .unwrap();
    }

    #[test]
    fn deserialize_with_new_field_order_works_with_bin_file_recorder() {
        deserialize_with_new_field_order("bin", BinFileRecorder::<FullPrecisionSettings>::new())
            .unwrap();
    }

    fn deserialize_with_new_optional_field<R>(name: &str, recorder: R) -> Result<(), RecorderError>
    where
        R: FileRecorder,
    {
        let file_path: PathBuf = format!("/tmp/deserialize_with_new_optional_field-{name}").into();
        let model = Model {
            linear1: nn::LinearConfig::new(20, 20).init::<TestBackend>(),
            linear2: nn::LinearConfig::new(20, 20).init::<TestBackend>(),
        };

        recorder
            .record(model.into_record(), file_path.clone())
            .unwrap();
        let result = recorder.load::<ModelNewOptionalFieldRecord<TestBackend>>(file_path.clone());
        std::fs::remove_file(file_path).ok();

        result?;
        Ok(())
    }

    fn deserialize_with_new_field_order<R>(name: &str, recorder: R) -> Result<(), RecorderError>
    where
        R: FileRecorder,
    {
        let file_path: PathBuf = format!("/tmp/deserialize_with_new_field_order-{name}").into();
        let model = Model {
            linear1: nn::LinearConfig::new(20, 20).init::<TestBackend>(),
            linear2: nn::LinearConfig::new(20, 20).init::<TestBackend>(),
        };

        recorder
            .record(model.into_record(), file_path.clone())
            .unwrap();

        let result = recorder.load::<ModelNewFieldOrdersRecord<TestBackend>>(file_path.clone());
        std::fs::remove_file(file_path).ok();

        result?;
        Ok(())
    }
}
