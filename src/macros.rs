// Small little helper macro to create layouts, similar to the vec! macro in the standard library.
#[macro_export]
macro_rules! layout {
    ($( ($amount:expr, $t:ty, $kind:expr) ),*) => {
        {
            let mut l = $crate::ogl::buffers::array::Layout::default();

            $(
                l.push::<$t>($amount, $kind);
            )*

            l
        }
    }
}
