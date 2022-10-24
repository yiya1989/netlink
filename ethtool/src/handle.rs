// SPDX-License-Identifier: MIT

use futures::{future::Either, FutureExt, Stream, StreamExt, TryStream};
use genetlink::GenetlinkHandle;
use log::info;
use netlink_packet_core::{NetlinkMessage, NLM_F_ACK, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_generic::GenlMessage;
use netlink_packet_utils::DecodeError;

use crate::{
    try_ethtool, EthtoolChannelHandle, EthtoolCoalesceHandle, EthtoolError, EthtoolFeatureHandle,
    EthtoolLinkModeHandle, EthtoolMessage, EthtoolPauseHandle, EthtoolRingHandle,
};

#[derive(Clone, Debug)]
pub struct EthtoolHandle {
    pub handle: GenetlinkHandle,
}

impl EthtoolHandle {
    pub(crate) fn new(handle: GenetlinkHandle) -> Self {
        EthtoolHandle { handle }
    }

    pub fn pause(&mut self) -> EthtoolPauseHandle {
        EthtoolPauseHandle::new(self.clone())
    }

    pub fn feature(&mut self) -> EthtoolFeatureHandle {
        EthtoolFeatureHandle::new(self.clone())
    }

    pub fn link_mode(&mut self) -> EthtoolLinkModeHandle {
        EthtoolLinkModeHandle::new(self.clone())
    }

    pub fn ring(&mut self) -> EthtoolRingHandle {
        EthtoolRingHandle::new(self.clone())
    }

    pub fn coalesce(&mut self) -> EthtoolCoalesceHandle {
        EthtoolCoalesceHandle::new(self.clone())
    }

    pub fn channel(&mut self) -> EthtoolChannelHandle {
        EthtoolChannelHandle::new(self.clone())
    }

    pub async fn request(
        &mut self,
        message: NetlinkMessage<GenlMessage<EthtoolMessage>>,
    ) -> Result<
        impl Stream<Item = Result<NetlinkMessage<GenlMessage<EthtoolMessage>>, DecodeError>>,
        EthtoolError,
    > {
        self.handle
            .request(message)
            .await
            .map_err(|e| EthtoolError::RequestFailed(format!("BUG: Request failed with {}", e)))
    }
}

pub(crate) async fn ethtool_execute(
    handle: &mut EthtoolHandle,
    is_dump: bool,
    ethtool_msg: EthtoolMessage,
    with_ack: bool,
) -> impl TryStream<Ok = GenlMessage<EthtoolMessage>, Error = EthtoolError> {
    let mut nl_header_flags = if is_dump {
        // The NLM_F_ACK is required due to bug of kernel:
        //  https://bugzilla.redhat.com/show_bug.cgi?id=1953847
        // without `NLM_F_MULTI`, rust-netlink will not parse
        // multiple netlink message in single socket reply.
        // Using NLM_F_ACK will force rust-netlink to parse all till
        // acked at the end.
        NLM_F_DUMP | NLM_F_REQUEST | NLM_F_ACK
    } else {
        NLM_F_REQUEST
    };

    if with_ack {
        nl_header_flags |= NLM_F_ACK
    }

    info!("ethtool_execute: {:#?}", ethtool_msg);

    let mut nl_msg = NetlinkMessage::from(GenlMessage::from_payload(ethtool_msg));

    nl_msg.header.flags = nl_header_flags;

    info!("handle.request: {:#?}", nl_msg);

    match handle.request(nl_msg).await {
        Ok(response) => Either::Left(response.map(move |msg| Ok(try_ethtool!(msg)))),
        Err(e) => Either::Right(
            futures::future::err::<GenlMessage<EthtoolMessage>, EthtoolError>(e).into_stream(),
        ),
    }
}
