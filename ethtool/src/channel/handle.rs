// SPDX-License-Identifier: MIT

use log::info;

use crate::{EthtoolChannelGetRequest, EthtoolChannelSetRequest, EthtoolHandle};

pub struct EthtoolChannelHandle(EthtoolHandle);

impl EthtoolChannelHandle {
    pub fn new(handle: EthtoolHandle) -> Self {
        info!("EthtoolChannelHandle 1");
        EthtoolChannelHandle(handle)
    }

    /// Retrieve the ethtool features of a interface (equivalent to `ethtool -k eth1`)
    pub fn get(&mut self, iface_name: Option<&str>) -> EthtoolChannelGetRequest {
        EthtoolChannelGetRequest::new(self.0.clone(), iface_name)
    }

    /// Retrieve the ethtool features of a interface (equivalent to `ethtool -k eth1`)
    pub fn set(
        &mut self,
        iface_name: Option<&str>,
        combined_count: u32,
    ) -> EthtoolChannelSetRequest {
        EthtoolChannelSetRequest::new(self.0.clone(), iface_name, combined_count)
    }
}
