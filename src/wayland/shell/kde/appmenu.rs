//! KDE Appmenu protocol
//!
//! This interface allows a client to link a window (or wl_surface) to an com.canonical.dbusmenu
//! interface registered on DBus.
//!
//! ```
//! extern crate wayland_server;
//! extern crate smithay;
//!
//! use smithay::delegate_kde_appmenu;
//! use smithay::wayland::shell::kde::appmenu::{KdeAppMenuHandler, KdeAppMenuState};
//!
//! # struct State { kde_appmenu_state: KdeAppMenuState };
//! # let mut display = wayland_server::Display::<State>::new().unwrap();
//!
//! // Create the new KdeAppMenuState.
//! let state = KdeAppMenuState::new::<State>(&display.handle());
//!
//! // Insert KdeAppMenuState into your compositor state.
//! // …
//!
//! // Implement KDE appmenu handlers.
//! impl KdeAppMenuHandler for State {
//!     fn kde_appmenu_state(&self) -> &KdeAppMenuState {
//!         &self.kde_appmenu_state
//!     }
//! }
//!
//! delegate_kde_appmenu!(State);
//! ```
use wayland_protocols_plasma::appmenu::server::org_kde_kwin_appmenu::{
    OrgKdeKwinAppmenu, Request as AppmenuRequest,
};
use wayland_protocols_plasma::appmenu::server::org_kde_kwin_appmenu_manager::{
    OrgKdeKwinAppmenuManager, Request as ManagerRequest,
};
use wayland_server::backend::GlobalId;
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New};

/// KDE appmenu handler.
pub trait KdeAppMenuHandler {
    /// Return the KDE appmenu state.
    fn kde_appmenu_state(&self) -> &KdeAppMenuState;

    /// Handle new appmenu object creation.
    ///
    /// Called whenever a new appmenu object is created, usually this happens when a new window
    /// is opened.
    fn new_appmenu(&mut self, _surface: &WlSurface, _appmenu: &OrgKdeKwinAppmenu) {}

    /// Handle service name and object path being set.
    ///
    /// Called when a client provides the service name and object path for the appmenu.
    fn set_address(
        &mut self,
        _surface: &WlSurface,
        _appmenu: &OrgKdeKwinAppmenu,
        _service_name: String,
        _object_path: String,
    ) {
    }

    /// Handle appmenu object removal for a surface.
    fn release(&mut self, _appmenu: &OrgKdeKwinAppmenu, _surface: &WlSurface) {}
}

/// KDE appmenu state.
#[derive(Debug)]
pub struct KdeAppMenuState {
    kde_appmenu_manager: GlobalId,
}

/// Data associated with a KdeAppMenuManager global.
#[allow(missing_debug_implementations)]
pub struct KdeAppMenuManagerGlobalData {
    pub(crate) filter: Box<dyn for<'c> Fn(&'c Client) -> bool + Send + Sync>,
}

impl KdeAppMenuState {
    /// Create a new KDE appmenu global.
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<OrgKdeKwinAppmenuManager, KdeAppMenuManagerGlobalData>
            + Dispatch<OrgKdeKwinAppmenuManager, ()>
            + Dispatch<OrgKdeKwinAppmenu, WlSurface>
            + KdeAppMenuHandler
            + 'static,
    {
        Self::new_with_filter::<D, _>(display, |_| true)
    }

    /// Create a new KDE appmenu global with a filter.
    ///
    /// Filters can be used to limit visibility of a global to certain clients.
    pub fn new_with_filter<D, F>(display: &DisplayHandle, filter: F) -> Self
    where
        D: GlobalDispatch<OrgKdeKwinAppmenuManager, KdeAppMenuManagerGlobalData>
            + Dispatch<OrgKdeKwinAppmenuManager, ()>
            + Dispatch<OrgKdeKwinAppmenu, WlSurface>
            + KdeAppMenuHandler
            + 'static,
        F: for<'c> Fn(&'c Client) -> bool + Send + Sync + 'static,
    {
        let data = KdeAppMenuManagerGlobalData {
            filter: Box::new(filter),
        };
        let kde_appmenu_manager = display.create_global::<D, OrgKdeKwinAppmenuManager, _>(2, data);

        Self {
            kde_appmenu_manager,
        }
    }

    /// Returns the id of the [`OrgKdeKwinAppmenuManager`] global.
    pub fn global(&self) -> GlobalId {
        self.kde_appmenu_manager.clone()
    }
}

#[allow(missing_docs)] // TODO
#[macro_export]
macro_rules! delegate_kde_appmenu {
    ($(@<$( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+>)? $ty: ty) => {
        $crate::reexports::wayland_server::delegate_global_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols_plasma::appmenu::server::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager: $crate::wayland::shell::kde::appmenu::KdeAppMenuManagerGlobalData
        ] => $crate::wayland::shell::kde::appmenu::KdeAppMenuState);

        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols_plasma::appmenu::server::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager: ()
        ] => $crate::wayland::shell::kde::appmenu::KdeAppMenuState);

        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols_plasma::appmenu::server::org_kde_kwin_appmenu::OrgKdeKwinAppmenu: $crate::reexports::wayland_server::protocol::wl_surface::WlSurface
        ] => $crate::wayland::shell::kde::appmenu::KdeAppMenuState);
    };
}

impl<D> GlobalDispatch<OrgKdeKwinAppmenuManager, KdeAppMenuManagerGlobalData, D> for KdeAppMenuState
where
    D: GlobalDispatch<OrgKdeKwinAppmenuManager, KdeAppMenuManagerGlobalData>
        + Dispatch<OrgKdeKwinAppmenuManager, ()>
        + Dispatch<OrgKdeKwinAppmenu, WlSurface>
        + KdeAppMenuHandler
        + 'static,
{
    fn bind(
        _state: &mut D,
        _dh: &DisplayHandle,
        _client: &Client,
        resource: New<OrgKdeKwinAppmenuManager>,
        _global_data: &KdeAppMenuManagerGlobalData,
        data_init: &mut DataInit<'_, D>,
    ) {
        data_init.init(resource, ());
    }

    fn can_view(client: Client, global_data: &KdeAppMenuManagerGlobalData) -> bool {
        (global_data.filter)(&client)
    }
}

impl<D> Dispatch<OrgKdeKwinAppmenuManager, (), D> for KdeAppMenuState
where
    D: Dispatch<OrgKdeKwinAppmenuManager, ()>
        + Dispatch<OrgKdeKwinAppmenu, WlSurface>
        + KdeAppMenuHandler
        + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        _manager: &OrgKdeKwinAppmenuManager,
        request: ManagerRequest,
        _data: &(),
        _dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        let (id, surface) = match request {
            ManagerRequest::Create { id, surface } => (id, surface),
            _ => unreachable!(),
        };

        let appmenu = data_init.init(id, surface.clone());
        state.new_appmenu(&surface, &appmenu);
    }
}

impl<D> Dispatch<OrgKdeKwinAppmenu, WlSurface, D> for KdeAppMenuState
where
    D: Dispatch<OrgKdeKwinAppmenu, WlSurface> + KdeAppMenuHandler + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        appmenu: &OrgKdeKwinAppmenu,
        request: AppmenuRequest,
        surface: &WlSurface,
        _dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        match request {
            AppmenuRequest::SetAddress {
                service_name,
                object_path,
            } => {
                state.set_address(surface, appmenu, service_name, object_path);
            }
            AppmenuRequest::Release => {
                state.release(appmenu, surface);
            }
            _ => unreachable!(),
        }
    }
}
