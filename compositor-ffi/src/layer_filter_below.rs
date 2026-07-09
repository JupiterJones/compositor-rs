use std::sync::Arc;

use value_box::{BorrowedPtr, OwnedPtr, ReturnBoxerResult};

use compositor::{Filter, FilterBelowLayer, Geometry, Layer};

#[unsafe(no_mangle)]
pub extern "C" fn compositor_filter_below_layer_new(
    filter: BorrowedPtr<Filter>,
    geometry: BorrowedPtr<Geometry>,
) -> OwnedPtr<Arc<dyn Layer>> {
    filter
        .with_clone_ok(|filter| {
            geometry.with_clone_ok(|geometry| {
                OwnedPtr::new(Arc::new(FilterBelowLayer::new(filter, geometry)) as Arc<dyn Layer>)
            })
        })
        .or_log(OwnedPtr::null())
}
