use alloc::vec::Vec;
use core::ops::Range;

use crate::{backend::Backend, tensor::Shape, Data, Distribution, ElementConversion};

/// Operations on float tensors.
pub trait TensorOps<B: Backend> {
    fn from_data<const D: usize>(
        data: Data<B::FloatElem, D>,
        device: &B::Device,
    ) -> B::TensorPrimitive<D>;
    fn random<const D: usize>(
        shape: Shape<D>,
        distribution: Distribution<B::FloatElem>,
        device: &B::Device,
    ) -> B::TensorPrimitive<D>;
    fn zeros<const D: usize>(shape: Shape<D>, device: &B::Device) -> B::TensorPrimitive<D> {
        Self::from_data(Data::zeros(shape), device)
    }
    fn ones<const D: usize>(shape: Shape<D>, device: &B::Device) -> B::TensorPrimitive<D> {
        Self::from_data(Data::ones(shape), device)
    }
    fn shape<const D: usize>(tensor: &B::TensorPrimitive<D>) -> Shape<D>;
    fn to_data<const D: usize>(tensor: &B::TensorPrimitive<D>) -> Data<B::FloatElem, D>;
    fn into_data<const D: usize>(tensor: B::TensorPrimitive<D>) -> Data<B::FloatElem, D>;
    fn device<const D: usize>(tensor: &B::TensorPrimitive<D>) -> B::Device;
    fn to_device<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        device: &B::Device,
    ) -> B::TensorPrimitive<D>;
    fn arange(range: Range<usize>, device: &B::Device) -> B::IntTensorPrimitive<1> {
        let shape = Shape::new([range.end - range.start]);
        let value = range
            .into_iter()
            .map(|i| (i as i64).elem())
            .collect::<Vec<B::IntElem>>();
        let data = Data::new(value, shape);
        B::int_from_data(data, device)
    }
    fn empty<const D: usize>(shape: Shape<D>, device: &B::Device) -> B::TensorPrimitive<D>;
    fn repeat<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
        times: usize,
    ) -> B::TensorPrimitive<D> {
        let mut shape = B::shape(&tensor);
        if shape.dims[dim] != 1 {
            panic!("Can only repeat dimension with dim=1");
        }
        shape.dims[dim] = times;

        let mut i = 0;
        let indexes_select_all = [0; D].map(|_| {
            let start = 0;
            let end = shape.dims[i];
            i += 1;
            start..end
        });

        let mut tensor_output = B::empty(shape, &B::device(&tensor));
        for i in 0..times {
            let mut indexes = indexes_select_all.clone();
            indexes[dim] = i..i + 1;
            tensor_output = B::index_assign(tensor_output, indexes, tensor.clone());
        }

        tensor_output
    }
    fn add<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn add_scalar<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::TensorPrimitive<D>;
    fn sub<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn sub_scalar<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::TensorPrimitive<D>;
    fn mul<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn mul_scalar<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::TensorPrimitive<D>;
    fn div<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn div_scalar<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::TensorPrimitive<D>;
    fn matmul<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn neg<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn transpose<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D> {
        Self::swap_dims(tensor, D - 2, D - 1)
    }
    fn swap_dims<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim1: usize,
        dim2: usize,
    ) -> B::TensorPrimitive<D>;
    fn reshape<const D1: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D1>,
        shape: Shape<D2>,
    ) -> B::TensorPrimitive<D2>;
    fn index_select<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        indexes: B::IntTensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn index_select_assign<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        indexes: B::IntTensorPrimitive<D>,
        value: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn index_select_dim<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
        indexes: B::IntTensorPrimitive<1>,
    ) -> B::TensorPrimitive<D>;
    fn index_select_dim_assign<const D1: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D1>,
        dim: usize,
        indexes: B::IntTensorPrimitive<1>,
        value: B::TensorPrimitive<D2>,
    ) -> B::TensorPrimitive<D1>;
    fn index<const D1: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D1>,
        indexes: [Range<usize>; D2],
    ) -> B::TensorPrimitive<D1>;
    fn index_assign<const D1: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D1>,
        indexes: [Range<usize>; D2],
        value: B::TensorPrimitive<D1>,
    ) -> B::TensorPrimitive<D1>;
    fn mask_scatter<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        mask: B::BoolTensorPrimitive<D>,
        source: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn mask_fill<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        mask: B::BoolTensorPrimitive<D>,
        value: B::FloatElem,
    ) -> B::TensorPrimitive<D>;
    fn equal<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::BoolTensorPrimitive<D>;
    fn equal_elem<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::BoolTensorPrimitive<D>;
    fn greater<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::BoolTensorPrimitive<D>;
    fn greater_elem<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::BoolTensorPrimitive<D>;
    fn greater_equal<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::BoolTensorPrimitive<D>;
    fn greater_equal_elem<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::BoolTensorPrimitive<D>;
    fn lower<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::BoolTensorPrimitive<D>;
    fn lower_elem<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::BoolTensorPrimitive<D>;
    fn lower_equal<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::TensorPrimitive<D>,
    ) -> B::BoolTensorPrimitive<D>;
    fn lower_equal_elem<const D: usize>(
        lhs: B::TensorPrimitive<D>,
        rhs: B::FloatElem,
    ) -> B::BoolTensorPrimitive<D>;
    fn detach<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D> {
        // Should only be overriden by autodiff backends.
        tensor
    }
    fn set_require_grad<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        _require_grad: bool,
    ) -> B::TensorPrimitive<D> {
        // Should only be overriden by autodiff backends.
        tensor
    }
    fn is_require_grad<const D: usize>(_tensor: &B::TensorPrimitive<D>) -> bool {
        // Should only be overriden by autodiff backends.
        false
    }
    fn sum<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<1>;
    fn sum_dim<const D: usize>(tensor: B::TensorPrimitive<D>, dim: usize) -> B::TensorPrimitive<D>;
    fn mean<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<1>;
    fn mean_dim<const D: usize>(tensor: B::TensorPrimitive<D>, dim: usize)
        -> B::TensorPrimitive<D>;
    fn to_full_precision<const D: usize>(
        tensor: &B::TensorPrimitive<D>,
    ) -> <B::FullPrecisionBackend as Backend>::TensorPrimitive<D>;
    fn from_full_precision<const D: usize>(
        tensor: <B::FullPrecisionBackend as Backend>::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn exp<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn log<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn log1p<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn powf<const D: usize>(tensor: B::TensorPrimitive<D>, value: f32) -> B::TensorPrimitive<D>;
    fn sqrt<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn cos<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn sin<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn tanh<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn erf<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<D>;
    fn cat<const D: usize>(
        tensors: Vec<B::TensorPrimitive<D>>,
        dim: usize,
    ) -> B::TensorPrimitive<D>;
    fn argmax<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
    ) -> B::IntTensorPrimitive<D>;
    fn argmin<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
    ) -> B::IntTensorPrimitive<D>;

    fn max<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<1> {
        let shape = B::shape(&tensor);
        let tensor = B::reshape(tensor, Shape::new([shape.num_elements()]));

        B::max_dim(tensor, 0)
    }
    fn max_dim<const D: usize>(tensor: B::TensorPrimitive<D>, dim: usize) -> B::TensorPrimitive<D> {
        let index = B::argmax(tensor.clone(), dim);

        B::index_select(tensor, index)
    }
    fn max_dim_with_indexes<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
    ) -> (B::TensorPrimitive<D>, B::IntTensorPrimitive<D>) {
        let index = B::argmax(tensor.clone(), dim);
        let values = B::index_select(tensor, index.clone());

        (values, index)
    }
    fn min<const D: usize>(tensor: B::TensorPrimitive<D>) -> B::TensorPrimitive<1> {
        let shape = B::shape(&tensor);
        let tensor = B::reshape(tensor, Shape::new([shape.num_elements()]));

        B::min_dim(tensor, 0)
    }
    fn min_dim<const D: usize>(tensor: B::TensorPrimitive<D>, dim: usize) -> B::TensorPrimitive<D> {
        let index = B::argmin(tensor.clone(), dim);

        B::index_select(tensor, index)
    }
    fn min_dim_with_indexes<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
    ) -> (B::TensorPrimitive<D>, B::IntTensorPrimitive<D>) {
        let index = B::argmin(tensor.clone(), dim);
        let values = B::index_select(tensor, index.clone());

        (values, index)
    }
    fn unbind<const D: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
    ) -> Vec<B::TensorPrimitive<D2>>;
    fn cumsum<const D: usize>(tensor: B::TensorPrimitive<D>, dim: usize) -> B::TensorPrimitive<D>;
    fn stack<const D: usize, const D2: usize>(
        tensors: Vec<B::TensorPrimitive<D>>,
        dim: usize,
    ) -> B::TensorPrimitive<D2>;
    fn narrow<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: usize,
        start: usize,
        length: usize,
    ) -> B::TensorPrimitive<D>;
    fn upsample_linear1d<const D: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D>,
        output_size: &[usize],
        align_corners: bool,
        scales: impl Into<Option<f64>>,
    ) -> B::TensorPrimitive<D2>;
    fn pad<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        pad: &[usize],
        mode: &str,
        value: impl Into<Option<f64>>,
    ) -> B::TensorPrimitive<D>;
    fn expand<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        size: Vec<usize>,
        implicit: bool,
    ) -> B::TensorPrimitive<D>;
    fn upsample_bilinear2d<const D: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D>,
        output_size: Vec<usize>,
        align_corners: bool,
        scales_h: impl Into<Option<f64>>,
        scales_w: impl Into<Option<f64>>,
    ) -> B::TensorPrimitive<D2>;
    fn select<const D: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D>,
        dim: i64,
        index: i64,
    ) -> B::TensorPrimitive<D2>;
    fn flip<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dims: Vec<usize>,
    ) -> B::TensorPrimitive<D>;
    fn permute<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        dims: [usize; D],
    ) -> B::TensorPrimitive<D>;
    fn einsum<const D: usize, const D2: usize, const D3: usize>(
        equation: &str,
        tensor1: B::TensorPrimitive<D>,
        tensor2: B::TensorPrimitive<D2>,
    ) -> B::TensorPrimitive<D3>;
    fn index_tch<const D: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D>,
        indices: Vec<B::IntTensorPrimitive<D>>,
    ) -> B::TensorPrimitive<D2>;
    fn repeat_interleave_self_int<const D: usize, const D2: usize>(
        tensor: B::TensorPrimitive<D>,
        repeats: usize,
        dim: Option<usize>,
        output_size: Option<usize>,
    ) -> B::TensorPrimitive<D2>;
    fn where_self<const D: usize>(
        tensor: B::TensorPrimitive<D>,
        condition: B::BoolTensorPrimitive<D>,
        other: B::TensorPrimitive<D>,
    ) -> B::TensorPrimitive<D>;
    fn copy_<const D:usize>(tensor: &mut B::TensorPrimitive<D>, src: B::TensorPrimitive<D>);
}
