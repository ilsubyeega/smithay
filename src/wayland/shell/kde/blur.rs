//! KDE window blur protocol
//!
//! This interface allows a compositor to announce support for KDE's blur protocol.
//!
//! A client can use this protocol to request blurring of its surface.
//!
//! ```
//! extern crate wayland_server;
//! extern crate smithay;
//!
//! use smithay::delegate_kde_blur;
//! use smithay::wayland::shell::kde::blur::{KdeBlurHandler, KdeBlurState};
//!
//! # struct State { kde_blur_state: KdeBlurState };
//! # let mut display = wayland_server::Display::<State>::new().unwrap();
//!
//! // Create the new KdeBlurState.
//! let state = KdeBlurState::new::<State>(&display.handle());
//!
//! // Insert KdeBlurState into your compositor state.
//! // …
//!
//! // Implement KDE blur handlers.
//! impl KdeBlurHandler for State {
//!     fn kde_blur_state(&self) -> &KdeBlurState {
//!         &self.kde_blur_state
//!     }
//! }
//!
//! delegate_kde_blur!(State);
//! ```

use wayland_protocols_plasma::blur::server::{
    org_kde_kwin_blur::OrgKdeKwinBlur,
    org_kde_kwin_blur_manager::OrgKdeKwinBlurManager,
};
use wayland_server::protocol::{wl_region::WlRegion, wl_surface::WlSurface};
use wayland_server::{backend::GlobalId, Client, Dispatch, DisplayHandle, GlobalDispatch};

/// KDE blur handler.
pub trait KdeBlurHandler {
    /// Return the KDE blur state.
    fn kde_blur_state(&self) -> &KdeBlurState;

    /// Handle new blur object creation.
    ///
    /// Called whenever a new blur object is created, usually this happens when a new window
    /// is opened.
    fn new_blur(&mut self, _surface: &WlSurface, _blur: &OrgKdeKwinBlur) {}

    /// Handle blur commit.
    ///
    /// Called when a client commits the blur state.
    fn commit(&mut self, _surface: &WlSurface, _blur: &OrgKdeKwinBlur) {}

    /// Handle setting the blur region.
    fn set_region(&mut self, _surface: &WlSurface, _blur: &OrgKdeKwinBlur, _region: Option<&WlRegion>) {}

    /// Handle blur object removal for a surface.
    fn release(&mut self, _blur: &OrgKdeKwinBlur, _surface: &WlSurface) {}

    /// Handle unsetting the blur.
    fn unset(&mut self, _surface: &WlSurface) {}
}

/// KDE blur state.
#[derive(Debug)]
pub struct KdeBlurState {
    kde_blur_manager: GlobalId,
}

/// Data associated with a KdeBlurManager global.
#[allow(missing_debug_implementations)]
pub struct KdeBlurManagerGlobalData {
    pub(crate) filter: Box<dyn for<'c> Fn(&'c Client) -> bool + Send + Sync>,
}

impl KdeBlurState {
    /// Create a new KDE blur global.
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<OrgKdeKwinBlurManager, KdeBlurManagerGlobalData>
            + Dispatch<OrgKdeKwinBlurManager, ()>
            + Dispatch<OrgKdeKwinBlur, WlSurface>
            + KdeBlurHandler
            + 'static,
    {
        Self::new_with_filter::<D, _>(display, |_| true)
    }

    /// Create a new KDE blur global with a filter.
    ///
    /// Filters can be used to limit visibility of a global to certain clients.
    pub fn new_with_filter<D, F>(display: &DisplayHandle, filter: F) -> Self
    where
        D: GlobalDispatch<OrgKdeKwinBlurManager, KdeBlurManagerGlobalData>
            + Dispatch<OrgKdeKwinBlurManager, ()>
            + Dispatch<OrgKdeKwinBlur, WlSurface>
            + KdeBlurHandler
            + 'static,
        F: for<'c> Fn(&'c Client) -> bool + Send + Sync + 'static,
    {
        let data = KdeBlurManagerGlobalData {
            filter: Box::new(filter),
        };
        let kde_blur_manager =
            display.create_global::<D, OrgKdeKwinBlurManager, _>(1, data);

        Self {
            kde_blur_manager,
        }
    }

    /// Returns the id of the [`OrgKdeKwinBlurManager`] global.
    pub fn global(&self) -> GlobalId {
        self.kde_blur_manager.clone()
    }
}

#[allow(missing_docs)] // TODO
#[macro_export]
macro_rules! delegate_kde_blur {
    ($(@<$( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+>)? $ty: ty) => {
        $crate::reexports::wayland_server::delegate_global_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols_plasma::blur::server::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager: $crate::wayland::shell::kde::blur::KdeBlurManagerGlobalData
        ] => $crate::wayland::shell::kde::blur::KdeBlurState);

        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols_plasma::blur::server::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager: ()
        ] => $crate::wayland::shell::kde::blur::KdeBlurState);

        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols_plasma::blur::server::org_kde_kwin_blur::OrgKdeKwinBlur: $crate::reexports::wayland_server::protocol::wl_surface::WlSurface
        ] => $crate::wayland::shell::kde::blur::KdeBlurState);
    };
}
