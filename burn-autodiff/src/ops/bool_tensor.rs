use crate::{
    tensor::{BoolTensor, IntTensor},
    ADBackendDecorator,
};

use burn_tensor::{backend::Backend, ops::BoolTensorOps, Data, Shape};

impl<B: Backend> BoolTensorOps<ADBackendDecorator<B>> for ADBackendDecorator<B> {
    fn bool_from_data<const D: usize>(data: Data<bool, D>, device: &B::Device) -> BoolTensor<B, D> {
        B::bool_from_data(data, device)
    }

    fn bool_shape<const D: usize>(tensor: &BoolTensor<B, D>) -> Shape<D> {
        B::bool_shape(tensor)
    }

    fn bool_to_data<const D: usize>(tensor: &BoolTensor<B, D>) -> Data<bool, D> {
        B::bool_to_data(tensor)
    }

    fn bool_into_data<const D: usize>(tensor: BoolTensor<B, D>) -> Data<bool, D> {
        B::bool_into_data(tensor)
    }

    fn bool_into_int<const D: usize>(tensor: BoolTensor<B, D>) -> IntTensor<B, D> {
        B::bool_into_int(tensor)
    }

    fn bool_to_device<const D: usize>(
        tensor: BoolTensor<B, D>,
        device: &B::Device,
    ) -> BoolTensor<B, D> {
        B::bool_to_device(tensor, device)
    }

    fn bool_device<const D: usize>(tensor: &BoolTensor<B, D>) -> B::Device {
        B::bool_device(tensor)
    }

    fn bool_reshape<const D1: usize, const D2: usize>(
        tensor: BoolTensor<B, D1>,
        shape: Shape<D2>,
    ) -> BoolTensor<B, D2> {
        B::bool_reshape(tensor, shape)
    }

    fn bool_index<const D1: usize, const D2: usize>(
        tensor: BoolTensor<B, D1>,
        indexes: [std::ops::Range<usize>; D2],
    ) -> BoolTensor<B, D1> {
        B::bool_index(tensor, indexes)
    }

    fn bool_empty<const D: usize>(
        shape: Shape<D>,
        device: &<ADBackendDecorator<B> as Backend>::Device,
    ) -> BoolTensor<B, D> {
        B::bool_empty(shape, device)
    }

    fn bool_index_assign<const D1: usize, const D2: usize>(
        tensor: <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D1>,
        indexes: [std::ops::Range<usize>; D2],
        value: <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D1>,
    ) -> <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D1> {
        B::bool_index_assign(tensor, indexes, value)
    }

    fn bool_cat<const D: usize>(tensors: Vec<BoolTensor<B, D>>, dim: usize) -> BoolTensor<B, D> {
        B::bool_cat(tensors, dim)
    }

    fn bool_equal<const D: usize>(
        lhs: BoolTensor<B, D>,
        rhs: BoolTensor<B, D>,
    ) -> BoolTensor<B, D> {
        B::bool_equal(lhs, rhs)
    }

    fn bool_equal_elem<const D: usize>(lhs: BoolTensor<B, D>, rhs: bool) -> BoolTensor<B, D> {
        B::bool_equal_elem(lhs, rhs)
    }
    fn bool_permute<const D: usize>(
        tensor: <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D>,
        dims: [usize; D],
    ) -> <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D> {
        unimplemented!()
    }
    fn bool_flip<const D: usize>(
        tensor: <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D>,
        dims: Vec<usize>,
    ) -> <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D> {
        unimplemented!()
    }
    fn bool_upsample_bilinear2d<const D: usize, const D2: usize>(
        tensor: <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D>,
        output_size: Vec<usize>,
        align_corners: bool,
        scales_h: impl Into<Option<f64>>,
        scales_w: impl Into<Option<f64>>,
    ) -> <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D2> {
        unimplemented!()
    }
    fn bool_select<const D: usize, const D2: usize>(
            tensor: <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D>,
            dim: i64,
            index: i64,
        ) -> <ADBackendDecorator<B> as Backend>::BoolTensorPrimitive<D2> {
        unimplemented!()
    }
}
