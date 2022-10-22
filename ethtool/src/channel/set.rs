// SPDX-License-Identifier: MIT

use futures::TryStream;
use netlink_packet_generic::GenlMessage;

use crate::{ethtool_execute, EthtoolError, EthtoolHandle, EthtoolMessage};

pub struct EthtoolChannelSetRequest {
    handle: EthtoolHandle,
    iface_name: Option<String>,
    combined_count: u32,
}

impl EthtoolChannelSetRequest {
    pub(crate) fn new(
        handle: EthtoolHandle,
        iface_name: Option<&str>,
        combined_count: u32,
    ) -> Self {
        EthtoolChannelSetRequest {
            handle,
            iface_name: iface_name.map(|i| i.to_string()),
            combined_count,
        }
    }

    pub async fn execute(
        self,
    ) -> impl TryStream<Ok = GenlMessage<EthtoolMessage>, Error = EthtoolError> {
        let EthtoolChannelSetRequest {
            mut handle,
            iface_name,
            combined_count,
        } = self;

        let ethtool_msg = EthtoolMessage::new_channel_set(iface_name.as_deref(), combined_count);
        ethtool_execute(&mut handle, iface_name.is_none(), ethtool_msg, true).await
    }
}
