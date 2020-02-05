// Copyright (c) 2019, Arm Limited, All Rights Reserved
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use super::generated_ops::key_attributes::{
    self, key_attributes_proto::AlgorithmProto, EccCurve, HashAlgorithm as HashAlgorithmProto,
    KeyAttributesProto,
};
use crate::operations::key_attributes::{Algorithm, AlgorithmInner, KeyAttributes};
use crate::requests::{ResponseStatus, Result};
use log::error;
use num::FromPrimitive;
use std::convert::{TryFrom, TryInto};

impl TryFrom<KeyAttributesProto> for KeyAttributes {
    type Error = ResponseStatus;

    fn try_from(attrs: KeyAttributesProto) -> Result<Self> {
        let key_type = FromPrimitive::from_i32(attrs.key_type).ok_or_else(|| {
            error!("Failed to convert key type");
            ResponseStatus::InvalidEncoding
        })?;
        let ecc_curve = match attrs.ecc_curve() {
            EccCurve::NoEccCurve => None,
            _ => Some(FromPrimitive::from_i32(attrs.ecc_curve).ok_or_else(|| {
                error!("Failed to convert ecc curve");
                ResponseStatus::InvalidEncoding
            })?),
        };

        let algorithm = attrs
            .algorithm_proto
            .ok_or_else(|| {
                error!("Algorithm was empty");
                ResponseStatus::InvalidEncoding
            })?
            .try_into()?;

        Ok(KeyAttributes {
            key_type,
            ecc_curve,
            algorithm,
            key_size: attrs.key_size,
            permit_export: attrs.permit_export,
            permit_encrypt: attrs.permit_encrypt,
            permit_decrypt: attrs.permit_decrypt,
            permit_sign: attrs.permit_sign,
            permit_verify: attrs.permit_verify,
            permit_derive: attrs.permit_derive,
        })
    }
}

impl TryFrom<KeyAttributes> for KeyAttributesProto {
    type Error = ResponseStatus;

    fn try_from(attrs: KeyAttributes) -> Result<Self> {
        Ok(KeyAttributesProto {
            key_type: attrs.key_type as i32,
            ecc_curve: match attrs.ecc_curve {
                None => 0,
                Some(curve) => curve as i32,
            },
            algorithm_proto: Some(attrs.algorithm.try_into()?),
            key_size: attrs.key_size,
            permit_export: attrs.permit_export,
            permit_encrypt: attrs.permit_encrypt,
            permit_decrypt: attrs.permit_decrypt,
            permit_sign: attrs.permit_sign,
            permit_verify: attrs.permit_verify,
            permit_derive: attrs.permit_derive,
        })
    }
}

impl TryFrom<AlgorithmProto> for Algorithm {
    type Error = ResponseStatus;

    fn try_from(alg: AlgorithmProto) -> Result<Self> {
        match alg {
            AlgorithmProto::Sign(sign) => Ok(Algorithm::sign(
                FromPrimitive::from_i32(sign.sign_algorithm).ok_or_else(|| {
                    error!("Failed to convert algorithm");
                    ResponseStatus::InvalidEncoding
                })?,
                match sign.hash_algorithm() {
                    HashAlgorithmProto::NoHashAlgorithm => None,
                    _ => Some(FromPrimitive::from_i32(sign.hash_algorithm).ok_or_else(|| {
                        error!("Failed to convert hash algorithm");
                        ResponseStatus::InvalidEncoding
                    })?),
                },
            )),
            _ => Err(ResponseStatus::PsaErrorNotSupported),
        }
    }
}

impl TryFrom<Algorithm> for AlgorithmProto {
    type Error = ResponseStatus;

    fn try_from(alg: Algorithm) -> Result<Self> {
        match alg.inner() {
            AlgorithmInner::Sign(sign, hash) => Ok(AlgorithmProto::Sign(key_attributes::Sign {
                sign_algorithm: *sign as i32,
                hash_algorithm: match hash {
                    None => 0,
                    Some(hash) => *hash as i32,
                },
            })),
            _ => Err(ResponseStatus::PsaErrorNotSupported),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::generated_ops::key_attributes::{
        self as key_attributes_proto, key_attributes_proto::AlgorithmProto, KeyAttributesProto,
    };
    use crate::operations::key_attributes::{self, Algorithm, AlgorithmInner, KeyAttributes};
    use std::convert::TryInto;

    #[test]
    fn key_attrs_to_proto() {
        let algo = Algorithm::sign(
            key_attributes::SignAlgorithm::RsaPkcs1v15Sign,
            Some(key_attributes::HashAlgorithm::Sha1),
        );
        let key_attrs = KeyAttributes {
            key_type: key_attributes::KeyType::RsaKeypair,
            ecc_curve: Some(key_attributes::EccCurve::Secp160k1),
            algorithm: algo,
            key_size: 1024,
            permit_export: true,
            permit_encrypt: true,
            permit_decrypt: true,
            permit_sign: true,
            permit_verify: true,
            permit_derive: true,
        };

        let key_attrs_proto: KeyAttributesProto = key_attrs.try_into().unwrap();

        assert_eq!(
            key_attrs_proto.key_type,
            key_attributes_proto::KeyType::RsaKeypair as i32
        );
        assert_eq!(
            key_attrs_proto.ecc_curve,
            key_attributes_proto::EccCurve::Secp160k1 as i32
        );
        assert_eq!(key_attrs_proto.key_size, 1024);
        assert!(key_attrs_proto.permit_export);
        assert!(key_attrs_proto.permit_encrypt);
        assert!(key_attrs_proto.permit_decrypt);
        assert!(key_attrs_proto.permit_sign);
        assert!(key_attrs_proto.permit_verify);
        assert!(key_attrs_proto.permit_derive);
    }

    #[test]
    fn key_attrs_from_proto() {
        let algo = Some(AlgorithmProto::Sign(key_attributes_proto::Sign {
            sign_algorithm: key_attributes_proto::SignAlgorithm::RsaPkcs1v15Sign as i32,
            hash_algorithm: key_attributes_proto::HashAlgorithm::Sha1 as i32,
        }));
        let key_attrs_proto = KeyAttributesProto {
            key_type: key_attributes_proto::KeyType::RsaKeypair as i32,
            ecc_curve: key_attributes_proto::EccCurve::Secp160k1 as i32,
            algorithm_proto: algo,
            key_size: 1024,
            permit_export: true,
            permit_encrypt: true,
            permit_decrypt: true,
            permit_sign: true,
            permit_verify: true,
            permit_derive: true,
        };

        let key_attrs: KeyAttributes = key_attrs_proto.try_into().unwrap();
        assert_eq!(key_attrs.key_type, key_attributes::KeyType::RsaKeypair);
        assert_eq!(
            key_attrs.ecc_curve,
            Some(key_attributes::EccCurve::Secp160k1)
        );
        assert_eq!(key_attrs.key_size, 1024);
        assert!(key_attrs.permit_decrypt);
        assert!(key_attrs.permit_encrypt);
        assert!(key_attrs.permit_sign);
        assert!(key_attrs.permit_verify);
        assert!(key_attrs.permit_derive);
        assert!(key_attrs.permit_export);
    }

    #[test]
    fn sign_algo_from_proto() {
        let proto_sign = AlgorithmProto::Sign(key_attributes_proto::Sign {
            sign_algorithm: key_attributes_proto::SignAlgorithm::RsaPkcs1v15Sign as i32,
            hash_algorithm: key_attributes_proto::HashAlgorithm::Sha1 as i32,
        });

        let sign: Algorithm = proto_sign.try_into().unwrap();

        assert_eq!(
            *sign.inner(),
            AlgorithmInner::Sign(
                key_attributes::SignAlgorithm::RsaPkcs1v15Sign,
                Some(key_attributes::HashAlgorithm::Sha1)
            )
        );
    }

    #[test]
    fn sign_algo_to_proto() {
        let sign = Algorithm::sign(
            key_attributes::SignAlgorithm::RsaPkcs1v15Sign,
            Some(key_attributes::HashAlgorithm::Sha1),
        );

        let proto_sign: AlgorithmProto = sign.try_into().unwrap();

        assert_eq!(
            proto_sign,
            AlgorithmProto::Sign(key_attributes_proto::Sign {
                sign_algorithm: key_attributes_proto::SignAlgorithm::RsaPkcs1v15Sign as i32,
                hash_algorithm: key_attributes_proto::HashAlgorithm::Sha1 as i32,
            })
        );
    }
}
