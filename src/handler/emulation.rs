use chromiumoxide_cdp::cdp::browser_protocol::emulation::{
    ScreenOrientation, ScreenOrientationType, SetDeviceMetricsOverrideParams,
    SetGeolocationOverrideParams, SetHardwareConcurrencyOverrideParams, SetTimezoneOverrideParams,
    SetTouchEmulationEnabledParams, SetUserAgentOverrideParams,
};
use chromiumoxide_types::{Command, MethodId};

use crate::cmd::CommandChain;
use crate::handler::viewport::Viewport;
use std::time::Duration;

#[derive(Debug)]
pub struct EmulationManager {
    pub emulating_mobile: bool,
    pub has_touch: bool,
    pub needs_reload: bool,
    pub request_timeout: Duration,
}

impl EmulationManager {
    pub fn new(request_timeout: Duration) -> Self {
        Self {
            emulating_mobile: false,
            has_touch: false,
            needs_reload: false,
            request_timeout,
        }
    }

    pub fn init_commands(&mut self, overrides: &EmulationOverrides) -> Option<CommandChain> {
        let mut cmds: Vec<(MethodId, serde_json::Value)> = Vec::with_capacity(10);

        if let Some(user_agent) = &overrides.user_agent {
            cmds.push(user_agent.to_cmd());
        }
        if let Some(hardware_concurrency) = &overrides.hardware_concurrency {
            cmds.push(hardware_concurrency.to_cmd());
        }
        if let Some(timezone) = &overrides.timezone {
            cmds.push(timezone.to_cmd());
        }
        if let Some(geolocation) = &overrides.geolocation {
            cmds.push(geolocation.to_cmd());
        }
        if let Some(viewport) = &overrides.viewport {
            let orientation = if viewport.is_landscape {
                ScreenOrientation::new(ScreenOrientationType::LandscapePrimary, 90)
            } else {
                ScreenOrientation::new(ScreenOrientationType::PortraitPrimary, 0)
            };
            let device_metrics = SetDeviceMetricsOverrideParams::builder()
                .mobile(viewport.emulating_mobile)
                .width(viewport.width)
                .height(viewport.height)
                .device_scale_factor(viewport.device_scale_factor.unwrap_or(1.))
                .screen_orientation(orientation)
                .build()
                .unwrap();
            let mut touch_emulation = SetTouchEmulationEnabledParams::new(viewport.has_touch);
            touch_emulation.max_touch_points = viewport.max_touch_points;

            cmds.push(device_metrics.to_cmd());
            cmds.push(touch_emulation.to_cmd());

            self.needs_reload = self.emulating_mobile != viewport.emulating_mobile
                || self.has_touch != viewport.has_touch;
        }

        if cmds.is_empty() {
            None
        } else {
            Some(CommandChain::new(cmds, self.request_timeout))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmulationOverrides {
    pub user_agent: Option<SetUserAgentOverrideParams>,
    pub hardware_concurrency: Option<SetHardwareConcurrencyOverrideParams>,
    pub timezone: Option<SetTimezoneOverrideParams>,
    pub geolocation: Option<SetGeolocationOverrideParams>,
    pub viewport: Option<Viewport>,
}

trait ToCommandChainItem {
    fn to_cmd(&self) -> (MethodId, serde_json::Value);
}

impl<T: Command> ToCommandChainItem for T {
    fn to_cmd(&self) -> (MethodId, serde_json::Value) {
        (self.identifier(), serde_json::to_value(self).unwrap())
    }
}
