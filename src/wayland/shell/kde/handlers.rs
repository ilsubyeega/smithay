//! Handlers for KDE decoration events.
use tracing::trace;

use wayland_protocols_plasma::blur::server::{
    org_kde_kwin_blur::{OrgKdeKwinBlur, Request},
    org_kde_kwin_blur_manager::{OrgKdeKwinBlurManager, Request as ManagerRequest},
};
use wayland_protocols_misc::server_decoration::server::{
    org_kde_kwin_server_decoration::{
        OrgKdeKwinServerDecoration, Request as DecorationRequest,
    },
    org_kde_kwin_server_decoration_manager::{
        OrgKdeKwinServerDecorationManager, Request as DecorationManagerRequest,
    },
};
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource};

use crate::wayland::shell::kde::{
    blur::{KdeBlurHandler, KdeBlurState},
    decoration::{KdeDecorationHandler, KdeDecorationState},
};

use super::{blur::KdeBlurManagerGlobalData, decoration::KdeDecorationManagerGlobalData};

impl<D> GlobalDispatch<OrgKdeKwinServerDecorationManager, KdeDecorationManagerGlobalData, D>
    for KdeDecorationState
where
    D: GlobalDispatch<OrgKdeKwinServerDecorationManager, KdeDecorationManagerGlobalData>
        + Dispatch<OrgKdeKwinServerDecorationManager, ()>
        + Dispatch<OrgKdeKwinServerDecoration, WlSurface>
        + KdeDecorationHandler
        + 'static,
{
    fn bind(
        state: &mut D,
        _dh: &DisplayHandle,
        _client: &Client,
        resource: New<OrgKdeKwinServerDecorationManager>,
        _global_data: &KdeDecorationManagerGlobalData,
        data_init: &mut DataInit<'_, D>,
    ) {
        let kde_decoration_manager = data_init.init(resource, ());

        // Set default decoration mode.
        let default_mode = state.kde_decoration_state().default_mode;
        kde_decoration_manager.default_mode(default_mode);

        trace!("Bound decoration manager global");
    }

    fn can_view(client: Client, global_data: &KdeDecorationManagerGlobalData) -> bool {
        (global_data.filter)(&client)
    }
}

impl<D> Dispatch<OrgKdeKwinBlur, WlSurface, D> for KdeBlurState
where
    D: Dispatch<OrgKdeKwinBlur, WlSurface> + KdeBlurHandler + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        blur: &OrgKdeKwinBlur,
        request: Request,
        surface: &WlSurface,
        _dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        trace!(
            surface = ?surface,
            request = ?request,
            "Blur request for surface"
        );

        match request {
            Request::Commit => state.commit(surface, blur),
            Request::SetRegion { region } => state.set_region(surface, blur, region.as_ref()),
            Request::Release => state.release(blur, surface),
            _ => unreachable!(),
        }
    }
}

impl<D> Dispatch<OrgKdeKwinBlurManager, (), D> for KdeBlurState
where
    D: Dispatch<OrgKdeKwinBlurManager, ()>
        + Dispatch<OrgKdeKwinBlur, WlSurface>
        + KdeBlurHandler
        + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        _blur_manager: &OrgKdeKwinBlurManager,
        request: ManagerRequest,
        _data: &(),
        _dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        match request {
            ManagerRequest::Create { id, surface } => {
                let blur = data_init.init(id, surface);

                let surface = blur.data().unwrap();
                state.new_blur(surface, &blur);

                trace!(surface = ?surface, "Created blur object for surface");
            }
            ManagerRequest::Unset { surface } => {
                state.unset(&surface);

                trace!(surface = ?surface, "Unset blur for surface");
            }
            _ => unreachable!(),
        }
    }
}

impl<D> GlobalDispatch<OrgKdeKwinBlurManager, KdeBlurManagerGlobalData, D> for KdeBlurState
where
    D: GlobalDispatch<OrgKdeKwinBlurManager, KdeBlurManagerGlobalData>
        + Dispatch<OrgKdeKwinBlurManager, ()>
        + Dispatch<OrgKdeKwinBlur, WlSurface>
        + KdeBlurHandler
        + 'static,
{
    fn bind(
        _state: &mut D,
        _dh: &DisplayHandle,
        _client: &Client,
        resource: New<OrgKdeKwinBlurManager>,
        _global_data: &KdeBlurManagerGlobalData,
        data_init: &mut DataInit<'_, D>,
    ) {
        data_init.init(resource, ());

        trace!("Bound blur manager global");
    }

    fn can_view(client: Client, global_data: &KdeBlurManagerGlobalData) -> bool {
        (global_data.filter)(&client)
    }
}

impl<D> Dispatch<OrgKdeKwinServerDecorationManager, (), D> for KdeDecorationState
where
    D: Dispatch<OrgKdeKwinServerDecorationManager, ()>
        + Dispatch<OrgKdeKwinServerDecorationManager, ()>
        + Dispatch<OrgKdeKwinServerDecoration, WlSurface>
        + KdeDecorationHandler
        + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        _kde_decoration_manager: &OrgKdeKwinServerDecorationManager,
        request: DecorationManagerRequest,
        _data: &(),
        _dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        let (id, surface) = match request {
            DecorationManagerRequest::Create { id, surface } => (id, surface),
            _ => unreachable!(),
        };

        let kde_decoration = data_init.init(id, surface);

        let surface = kde_decoration.data().unwrap();
        state.new_decoration(surface, &kde_decoration);

        trace!(surface = ?surface, "Created decoration object for surface");
    }
}

impl<D> Dispatch<OrgKdeKwinServerDecoration, WlSurface, D> for KdeDecorationState
where
    D: Dispatch<OrgKdeKwinServerDecoration, WlSurface> + KdeDecorationHandler + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        kde_decoration: &OrgKdeKwinServerDecoration,
        request: DecorationRequest,
        surface: &WlSurface,
        _dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        trace!(
            surface = ?surface,
            request = ?request,
            "Decoration request for surface"
        );

        match request {
            DecorationRequest::RequestMode { mode } => state.request_mode(surface, kde_decoration, mode),
            DecorationRequest::Release => state.release(kde_decoration, surface),
            _ => unreachable!(),
        }
    }
}
