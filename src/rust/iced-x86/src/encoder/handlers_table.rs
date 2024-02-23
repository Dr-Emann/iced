// SPDX-License-Identifier: MIT
// Copyright (C) 2018-present iced project and contributors

use crate::encoder::encoder_data::{ENC_FLAGS1, ENC_FLAGS2, ENC_FLAGS3};
use crate::encoder::enums::*;
use crate::encoder::op_code_handler::*;
use crate::enums::EncodingKind;
use crate::iced_constants::IcedConstants;
use crate::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::mem;
use lazy_static::lazy_static;

lazy_static! {
	pub(crate) static ref HANDLERS_TABLE: Box<[&'static OpCodeHandler; IcedConstants::CODE_ENUM_COUNT]> = {
		let mut v = Vec::with_capacity(IcedConstants::CODE_ENUM_COUNT);
		let invalid_handler = InvalidHandler::new().leaked();
		for code in Code::values() {
			let enc_flags1 = ENC_FLAGS1[code as usize];
			let enc_flags2 = ENC_FLAGS2[code as usize];
			let enc_flags3 = ENC_FLAGS3[code as usize];
			// SAFETY: The table is generated and only contains valid enum variants
			let encoding: EncodingKind = unsafe { mem::transmute(((enc_flags3 >> EncFlags3::ENCODING_SHIFT) & EncFlags3::ENCODING_MASK) as EncodingKindUnderlyingType) };
			let handler = match encoding {
				EncodingKind::Legacy => {
					if code == Code::INVALID {
						invalid_handler
					} else if code <= Code::DeclareQword {
						DeclareDataHandler::new(code).leaked()
					} else if code == Code::Zero_bytes {
						ZeroBytesHandler::new(code).leaked()
					} else {
						LegacyHandler::new(enc_flags1, enc_flags2, enc_flags3).leaked()
					}
				}
				#[cfg(not(feature = "no_vex"))]
				EncodingKind::VEX => VexHandler::new(enc_flags1, enc_flags2, enc_flags3).leaked(),
				#[cfg(feature = "no_vex")]
				EncodingKind::VEX => invalid_handler,
				#[cfg(not(feature = "no_evex"))]
				EncodingKind::EVEX => EvexHandler::new(enc_flags1, enc_flags2, enc_flags3).leaked(),
				#[cfg(feature = "no_evex")]
				EncodingKind::EVEX => invalid_handler,
				#[cfg(not(feature = "no_xop"))]
				EncodingKind::XOP => XopHandler::new(enc_flags1, enc_flags2, enc_flags3).leaked(),
				#[cfg(feature = "no_xop")]
				EncodingKind::XOP => invalid_handler,
				#[cfg(not(feature = "no_d3now"))]
				EncodingKind::D3NOW => D3nowHandler::new(enc_flags2, enc_flags3).leaked(),
				#[cfg(feature = "no_d3now")]
				EncodingKind::D3NOW => invalid_handler,
				#[cfg(feature = "mvex")]
				EncodingKind::MVEX => MvexHandler::new(enc_flags1, enc_flags2, enc_flags3).leaked(),
				#[cfg(not(feature = "mvex"))]
				EncodingKind::MVEX => invalid_handler,
			};
			v.push(handler);
		}
		#[allow(clippy::unwrap_used)]
		v.into_boxed_slice().try_into().ok().unwrap()
	};
}
