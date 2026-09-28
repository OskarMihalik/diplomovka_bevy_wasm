//! Generic tracking of api requests, keyed by the event that started them.
//!
//! Usage for a request started by `MyEvent`:
//! - register it once: `app.track_api::<MyEvent>()`
//! - send it with [`send_tracked::<MyEvent>`] instead of `client.send`
//! - in a UI system add an [`ApiStatus<MyEvent>`] param and use
//!   [`ApiStatus::is_pending`] / [`ApiStatus::succeeded`]
//!
//! Errors are still shown by the request's own handlers (`on_json_response` / `on_error`).
//! A failed request simply stops being pending without succeeding.
//!
//! Pending state is not stored anywhere, it is derived from the request entities
//! that `bevy_mod_reqwest` spawns, so it can't get stuck and needs no resetting.

use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_mod_reqwest::*;
use dto::default::ErrorDto;
use serde::de::IgnoredAny;

use crate::gui::gui::ShowErrorEvent;

/// Marker on the request entity, tells which event the request belongs to.
#[derive(Component)]
pub struct ApiRequest<E: Event>(PhantomData<E>);

impl<E: Event> Default for ApiRequest<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// The server answered the request started by `E` with `Ok`.
#[derive(Message)]
pub struct ApiSucceeded<E: Event>(PhantomData<E>);

pub trait TrackApiExt {
    fn track_api<E: Event>(&mut self) -> &mut Self;
}

impl TrackApiExt for App {
    fn track_api<E: Event>(&mut self) -> &mut Self {
        self.add_message::<ApiSucceeded<E>>()
    }
}

/// Like `client.send`, but marks the request as belonging to `E` and reports
/// success as [`ApiSucceeded<E>`].
/// Response handlers can still be added to the returned builder as usual.
pub fn send_tracked<'a, E: Event>(
    client: &'a mut BevyReqwest,
    commands: &mut Commands,
    request: reqwest::Request,
) -> BevyReqwestBuilder<'a> {
    let entity = commands
        .spawn((ApiRequest::<E>::default(), DespawnReqwestEntity))
        .id();

    client
        .send_using_entity(entity, request)
        .expect("request entity was just spawned")
        .on_response(report_response::<E>)
}

/// Every backend response is `Result<T, ErrorDto>`, so the outcome can be read
/// without knowing `T`. Parsing it here (and not via `on_json_response`) also
/// catches responses that are not valid json.
fn report_response<E: Event>(
    trigger: On<ReqwestResponseEvent>,
    mut commands: Commands,
    mut succeeded: MessageWriter<ApiSucceeded<E>>,
) {
    match trigger
        .event()
        .deserialize_json::<Result<IgnoredAny, ErrorDto>>()
    {
        Ok(Ok(_)) => {
            succeeded.write(ApiSucceeded(PhantomData));
        }
        // the request's own handler shows the error message
        Ok(Err(_)) => {}
        Err(error) => {
            // nobody else reports this, on_json_response only logs it
            commands.trigger(ShowErrorEvent {
                message: format!("Invalid response from server: {error}"),
            });
        }
    }
}

/// Status of requests started by `E`, for use in UI systems.
#[derive(SystemParam)]
pub struct ApiStatus<'w, 's, E: Event> {
    inflight: Query<'w, 's, (), (With<ApiRequest<E>>, With<ReqwestInflight>)>,
    succeeded: MessageReader<'w, 's, ApiSucceeded<E>>,
}

impl<E: Event> ApiStatus<'_, '_, E> {
    /// A request started by `E` is waiting for the server.
    pub fn is_pending(&self) -> bool {
        !self.inflight.is_empty()
    }

    /// A request started by `E` succeeded since the last call.
    pub fn succeeded(&mut self) -> bool {
        self.succeeded.read().count() > 0
    }
}
