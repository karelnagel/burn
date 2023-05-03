use crate as burn;

use crate::config::Config;
use crate::module::Module;
use crate::nn::conv::Conv2dPaddingConfig;
use crate::tensor::backend::Backend;
use crate::tensor::Tensor;
use burn_tensor::module::avg_pool2d;

/// Configuration to create an [2D avg pooling](AvgPool2d) layer.
#[derive(Config)]
pub struct AvgPool2dConfig {
    /// The number of channels.
    pub channels: usize,
    /// The size of the kernel.
    pub kernel_size: [usize; 2],
    /// The strides.
    #[config(default = "[1, 1]")]
    pub strides: [usize; 2],
    /// The padding configuration.
    #[config(default = "AvgPool2dPaddingConfig::Valid")]
    pub padding: AvgPool2dPaddingConfig,
}

/// Padding configuration for 2D avg pooling [config](AvgPool2dConfig).
pub type AvgPool2dPaddingConfig = Conv2dPaddingConfig;

/// Applies a 2D avg pooling over input tensors.
#[derive(Module, Debug, Clone)]
pub struct AvgPool2d {
    stride: [usize; 2],
    kernel_size: [usize; 2],
    padding: AvgPool2dPaddingConfig,
}

impl AvgPool2dConfig {
    /// Initialize a new [avg pool 2d](AvgPool2d) module.
    pub fn init(&self) -> AvgPool2d {
        AvgPool2d {
            stride: self.strides,
            kernel_size: self.kernel_size,
            padding: self.padding.clone(),
        }
    }
}

impl AvgPool2d {
    /// Applies the forward pass on the input tensor.
    ///
    /// # Shapes
    ///
    /// - input: [batch_size, channels, height_in, width_in],
    /// - output: [batch_size, channels, height_out, width_out],
    pub fn forward<B: Backend>(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let [_batch_size, _channels_in, height_in, width_in] = input.dims();
        let padding =
            self.padding
                .calculate_padding_2d(height_in, width_in, &self.kernel_size, &self.stride);

        avg_pool2d(input, self.kernel_size, self.stride, padding)
    }
}
