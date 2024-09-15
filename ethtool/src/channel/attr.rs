// SPDX-License-Identifier: MIT
use anyhow::Context;
use byteorder::{ByteOrder, NativeEndian};
use netlink_packet_utils::{
    nla::{DefaultNla, Nla, NlaBuffer, NlasIterator, NLA_F_NESTED},
    parsers::parse_u32,
    DecodeError, Emitable, Parseable,
};

use crate::{EthtoolAttr, EthtoolHeader};

#[allow(unused)]
const ETHTOOL_A_CHANNELS_UNSPEC: u16 = 0;
const ETHTOOL_A_CHANNELS_HEADER: u16 = 1;
#[allow(unused)]
const ETHTOOL_A_CHANNELS_RX_MAX: u16 = 2;
#[allow(dead_code)]
const ETHTOOL_A_CHANNELS_TX_MAX: u16 = 3;
#[allow(dead_code)]
const ETHTOOL_A_CHANNELS_OTHER_MAX: u16 = 4;
const ETHTOOL_A_CHANNELS_COMBINED_MAX: u16 = 5;
#[allow(dead_code)]
const ETHTOOL_A_CHANNELS_RX_COUNT: u16 = 6;
#[allow(dead_code)]
const ETHTOOL_A_CHANNELS_TX_COUNT: u16 = 7;
#[allow(dead_code)]
const ETHTOOL_A_CHANNELS_OTHER_COUNT: u16 = 8;
const ETHTOOL_A_CHANNELS_COMBINED_COUNT: u16 = 9;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EthtoolChannelAttr {
    Header(Vec<EthtoolHeader>),
    MaxCombined(u32),
    CombinedCount(u32),
    Other(DefaultNla),
}

impl Nla for EthtoolChannelAttr {
    fn value_len(&self) -> usize {
        match self {
            Self::Header(hdrs) => hdrs.as_slice().buffer_len(),
            Self::MaxCombined(_) | Self::CombinedCount(_) => 4,
            Self::Other(attr) => attr.value_len(),
        }
    }

    fn kind(&self) -> u16 {
        match self {
            Self::Header(_) => ETHTOOL_A_CHANNELS_HEADER | NLA_F_NESTED,
            Self::MaxCombined(_) => ETHTOOL_A_CHANNELS_COMBINED_MAX,
            Self::CombinedCount(_) => ETHTOOL_A_CHANNELS_COMBINED_COUNT,
            Self::Other(attr) => attr.kind(),
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        match self {
            Self::Header(ref nlas) => nlas.as_slice().emit(buffer),
            Self::MaxCombined(d) | Self::CombinedCount(d) => NativeEndian::write_u32(buffer, *d),
            Self::Other(ref attr) => attr.emit(buffer),
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NlaBuffer<&'a T>> for EthtoolChannelAttr {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        let payload = buf.value();
        Ok(match buf.kind() {
            ETHTOOL_A_CHANNELS_HEADER => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse channel header attributes";
                for nla in NlasIterator::new(payload) {
                    let nla = &nla.context(error_msg)?;
                    let parsed = EthtoolHeader::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                Self::Header(nlas)
            }
            ETHTOOL_A_CHANNELS_COMBINED_MAX => Self::MaxCombined(
                parse_u32(payload).context("Invalid ETHTOOL_A_CHANNELS_COMBINED_MAX value")?,
            ),
            ETHTOOL_A_CHANNELS_COMBINED_COUNT => Self::CombinedCount(
                parse_u32(payload).context("Invalid ETHTOOL_A_CHANNELS_COMBINED_COUNT value")?,
            ),
            _ => Self::Other(DefaultNla::parse(buf).context("invalid NLA (unknown kind)")?),
        })
    }
}

pub(crate) fn parse_channel_nlas(buffer: &[u8]) -> Result<Vec<EthtoolAttr>, DecodeError> {
    let mut nlas = Vec::new();
    for nla in NlasIterator::new(buffer) {
        let error_msg = format!(
            "Failed to parse ethtool channel message attribute {:?}",
            nla
        );
        let nla = &nla.context(error_msg.clone())?;
        let parsed = EthtoolChannelAttr::parse(nla).context(error_msg)?;
        nlas.push(EthtoolAttr::Channel(parsed));
    }
    Ok(nlas)
}
