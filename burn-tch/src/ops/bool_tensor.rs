use std::ops::Range;

use burn_tensor::{backend::Backend, ops::BoolTensorOps, Data, Shape};

use crate::{element::TchElement, TchBackend, TchDevice, TchTensor};

use super::TchOps;

impl<E: TchElement> BoolTensorOps<TchBackend<E>> for TchBackend<E> {
    fn bool_from_data<const D: usize>(
        data: Data<bool, D>,
        device: &TchDevice,
    ) -> TchTensor<bool, D> {
        TchTensor::from_data(data, (*device).into())
    }

    fn bool_shape<const D: usize>(tensor: &TchTensor<bool, D>) -> Shape<D> {
        tensor.shape()
    }

    fn bool_to_data<const D: usize>(tensor: &TchTensor<bool, D>) -> Data<bool, D> {
        let values: Vec<bool> = tensor.tensor.shallow_clone().into();
        Data::new(values, tensor.shape())
    }

    fn bool_into_data<const D: usize>(tensor: TchTensor<bool, D>) -> Data<bool, D> {
        let shape = tensor.shape();
        Data::new(tensor.tensor.into(), shape)
    }

    fn bool_to_device<const D: usize>(
        tensor: TchTensor<bool, D>,
        device: &TchDevice,
    ) -> TchTensor<bool, D> {
        TchTensor::new(tensor.tensor.to((*device).into()))
    }

    fn bool_reshape<const D1: usize, const D2: usize>(
        tensor: TchTensor<bool, D1>,
        shape: Shape<D2>,
    ) -> TchTensor<bool, D2> {
        TchOps::reshape(tensor, shape)
    }

    fn bool_device<const D: usize>(tensor: &TchTensor<bool, D>) -> TchDevice {
        tensor.tensor.device().into()
    }

    fn bool_empty<const D: usize>(
        shape: Shape<D>,
        device: &<TchBackend<E> as Backend>::Device,
    ) -> TchTensor<bool, D> {
        let tensor = tch::Tensor::empty(
            &shape.dims.map(|a| a as i64),
            (tch::Kind::Bool, (*device).into()),
        );

        TchTensor::new(tensor)
    }

    fn bool_index<const D1: usize, const D2: usize>(
        tensor: TchTensor<bool, D1>,
        indexes: [Range<usize>; D2],
    ) -> TchTensor<bool, D1> {
        TchOps::index(tensor, indexes)
    }
    fn bool_index_assign<const D1: usize, const D2: usize>(
        tensor: TchTensor<bool, D1>,
        indexes: [std::ops::Range<usize>; D2],
        value: TchTensor<bool, D1>,
    ) -> TchTensor<bool, D1> {
        TchOps::index_assign(tensor, indexes, value)
    }

    fn bool_cat<const D: usize>(
        tensors: Vec<TchTensor<bool, D>>,
        dim: usize,
    ) -> TchTensor<bool, D> {
        TchOps::cat(tensors, dim)
    }

    fn bool_equal<const D: usize>(
        lhs: TchTensor<bool, D>,
        rhs: TchTensor<bool, D>,
    ) -> TchTensor<bool, D> {
        TchOps::equal(lhs, rhs)
    }

    fn bool_equal_elem<const D: usize>(lhs: TchTensor<bool, D>, rhs: bool) -> TchTensor<bool, D> {
        let rhs = match rhs {
            true => 1,
            false => 0,
        };

        lhs.unary_ops(
            |mut tensor| tensor.eq_(rhs).to_kind(tch::Kind::Bool),
            |tensor| tensor.eq(rhs),
        )
    }

    fn bool_into_int<const D: usize>(tensor: TchTensor<bool, D>) -> TchTensor<i64, D> {
        let tensor = tensor.tensor.to_kind(E::KIND);
        TchTensor::new(tensor)
    }
    fn bool_permute<const D: usize>(
        tensor: <TchBackend<E> as Backend>::BoolTensorPrimitive<D>,
        dims: [usize; D],
    ) -> <TchBackend<E> as Backend>::BoolTensorPrimitive<D> {
        let dims = dims.iter().map(|x| *x as i64).collect::<Vec<_>>();
        tensor.unary_ops(
            |tensor| tensor.permute(&dims),
            |tensor| tensor.permute(&dims),
        )
    }
    fn bool_upsample_bilinear2d<const D: usize, const D2: usize>(
        tensor: TchTensor<bool, D>,
        output_size: Vec<usize>,
        align_corners: bool,
        scales_h: impl Into<Option<f64>>,
        scales_w: impl Into<Option<f64>>,
    ) -> TchTensor<bool, D2> {
        let output_size = output_size.iter().map(|x| *x as i64).collect::<Vec<_>>();
        let scales_h = scales_h.into();
        let scales_w = scales_w.into();
        tensor.unary_ops(
            |tensor| tensor.upsample_bilinear2d(&output_size, align_corners, scales_h, scales_w),
            |tensor| tensor.upsample_bilinear2d(&output_size, align_corners, scales_h, scales_w),
        )
    }

    fn bool_flip<const D: usize>(
        tensor: TchTensor<bool, D>,
        dims: Vec<usize>,
    ) -> TchTensor<bool, D> {
        let dims = dims.iter().map(|x| *x as i64).collect::<Vec<_>>();
        tensor.unary_ops(|tensor| tensor.flip(&dims), |tensor| tensor.flip(&dims))
    }
    fn bool_select<const D: usize, const D2: usize>(
            tensor: <TchBackend<E> as Backend>::BoolTensorPrimitive<D>,
            dim: i64,
            index: i64,
        ) -> <TchBackend<E> as Backend>::BoolTensorPrimitive<D2> {
            tensor.unary_ops(
                |tensor| tensor.select(dim as i64, index as i64),
                |tensor| tensor.select(dim as i64, index as i64),
            )
    }
}
