use bevy::prelude::*;
use bevy_mod_reqwest::*;
use dto::test::Test;

pub struct ApiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(send_requests);
    }
}

#[derive(Event)]
pub struct SendTestRequest {}

fn send_requests(_trigger: Trigger<SendTestRequest>, mut client: BevyReqwest) {
    let url = "http://localhost:4000/test";
    let body = Test {
        name: "aaaaaaa".to_string(),
        age: 10,
    };
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client.post(url).json(&body).build().unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(|trigger: Trigger<ReqwestResponseEvent>| {
            let response = trigger.event();
            let data = response.as_str();
            let status = response.status();
            // let headers = req.response_headers();
            bevy::log::info!("code: {status}, data: {data:?}");
        })
        // In case of request error, it can be reached using an observersystem as well
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            bevy::log::info!("error: {e:?}");
        });
}
