// Tracks collection and token volume (sum of the coin_amount's for a collection/token's buy/sell events)
// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use std::collections::HashMap;

use super::token_utils::{TokenDataIdType, TokenEvent};
use crate::{
    schema::{current_collection_volumes},
    util::{parse_timestamp},
};
use aptos_api_types::{Event as APIEvent, Transaction as APITransaction};
use bigdecimal::{BigDecimal, Zero};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(
    collection_data_id_hash
))]
#[diesel(table_name = current_collection_volumes)]
pub struct CurrentCollectionVolume {
    pub collection_data_id_hash: String,
    pub volume: BigDecimal,
    pub inserted_at: chrono::NaiveDateTime,
    pub last_transaction_version: i64,
}

struct TokenActivityHelper<'a> {
    pub token_data_id: &'a TokenDataIdType,
    pub property_version: BigDecimal,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_amount: BigDecimal,
    pub coin_type: Option<String>,
    pub coin_amount: Option<BigDecimal>,
}

impl CurrentCollectionVolume {
    pub fn from_transaction(transaction: &APITransaction) -> HashMap<String, Self> {
        let mut current_collection_volumes: HashMap<String, Self> = HashMap::new();
        if let APITransaction::UserTransaction(user_txn) = transaction {
            for event in &user_txn.events {
                let txn_version = user_txn.info.version.0 as i64;
                let event_type = event.typ.to_string();
                match TokenEvent::from_event(event_type.as_str(), &event.data, txn_version).unwrap()
                {
                    Some(token_event) => {
                        let parsed_event = Self::from_parse_event(
                            &event_type,
                            event,
                            &token_event,
                            txn_version,
                            parse_timestamp(user_txn.timestamp.0, txn_version),
                        );
                    if let Some(current_collection_volume) =  parsed_event {
                        current_collection_volumes.insert(
                            current_collection_volume.collection_data_id_hash.clone(), 
                            current_collection_volume.into()
                        )
                        } else {
                            None
                        }
                    }
                    None => None
                };
            }
        }
        current_collection_volumes
    }

    pub fn from_parse_event(
        event_type: &str,
        event: &APIEvent,
        token_event: &TokenEvent,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> Option<Self> {
        let event_account_address = &event.guid.account_address.to_string();
        let event_creation_number = event.guid.creation_number.0 as i64;
        let event_sequence_number = event.sequence_number.0 as i64;
        let binding = TokenDataIdType {
            creator: "".to_owned(),
            collection: "".to_owned(),
            name: "".to_owned(),
        }.clone();
        let token_data_id = match token_event {
            TokenEvent::BlueMoveAuctionEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueBidEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueBuyEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueChangePriceEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueClaimCoinsEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueClaimTokenEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueDelistEvent(inner) => &inner.id.token_data_id,
            TokenEvent::BlueListEvent(inner) => &inner.id.token_data_id,
            TokenEvent::TopazBidEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazBuyEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazCancelBidEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazClaimEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazDelistEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazListEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazSellEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::TopazSendEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::Souffl3BuyTokenEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::Souffl3CancelListTokenEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::Souffl3ListTokenEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::Souffl3TokenListEvent(inner) => &inner.token_id.token_data_id,
            TokenEvent::Souffl3TokenSwapEvent(inner) => &inner.token_id.token_data_id,
            _ => &binding
        };
        let binding = match token_event {
            TokenEvent::TopazCancelCollectionBidEvent(inner) => 
                TokenDataIdType {
                    creator: inner.creator.clone(),
                    collection: inner.collection_name.clone(),
                    name: "COLLECTION".to_owned(),
                }.clone(),
            TokenEvent::TopazCollectionBidEvent(inner) => 
                TokenDataIdType {
                    creator: inner.creator.clone(),
                    collection: inner.collection_name.clone(),
                    name: "COLLECTION".to_owned(),
                }.clone(),
            _ => TokenDataIdType {
                creator: "".to_owned(),
                collection: "".to_owned(),
                name: "COLLECTION".to_owned(),
            }.clone()
        };
        let token_activity_helper = match token_event {
            TokenEvent::MintTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id,
                property_version: BigDecimal::zero(),
                from_address: Some(event_account_address.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::BurnTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(event_account_address.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::MutateTokenPropertyMapEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.new_id.token_data_id,
                property_version: inner.new_id.property_version.clone(),
                from_address: Some(event_account_address.clone()),
                to_address: None,
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::WithdrawTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(event_account_address.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::DepositTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: None,
                to_address: Some((&event_account_address).to_string()),
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::OfferTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(event_account_address.clone()),
                to_address: Some(inner.to_address.clone()),
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::CancelTokenOfferEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(event_account_address.clone()),
                to_address: Some(inner.to_address.clone()),
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::ClaimTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(event_account_address.clone()),
                to_address: Some(inner.to_address.clone()),
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::BlueMoveAuctionEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(inner.owner_address.clone()),
                to_address: None,
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: Some(inner.min_selling_price.clone()),
            },
            TokenEvent::BlueBidEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(inner.bider_address.clone()),
                to_address: None,
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: Some(inner.bid.clone()),
            },
            TokenEvent::BlueBuyEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: None,
                to_address: Some(inner.buyer_address.clone()),
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::BlueChangePriceEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(inner.seller_address.clone()),
                to_address: None,
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: Some(inner.amount.clone()),
            },
            TokenEvent::BlueClaimCoinsEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(inner.owner_token.clone()),
                to_address: None,
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::BlueClaimTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: None,
                to_address: Some(inner.bider_address.clone()),
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::BlueDelistEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(inner.seller_address.clone()),
                to_address: None,
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::BlueListEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.id.token_data_id,
                property_version: inner.id.property_version.clone(),
                from_address: Some(inner.seller_address.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::TopazBidEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.buyer.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: Some(inner.coin_type.to_string()),
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazBuyEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.seller.clone()),
                to_address: Some(inner.buyer.clone()),
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazCancelBidEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.buyer.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: Some(inner.coin_type.to_string()),
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazCancelCollectionBidEvent(inner) => TokenActivityHelper {
                token_data_id: &binding,
                property_version: BigDecimal::zero(),
                from_address: Some(inner.buyer.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: Some(inner.coin_type.to_string()),
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazClaimEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: None,
                to_address: Some(inner.receiver.clone()),
                token_amount: BigDecimal::zero(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::TopazCollectionBidEvent(inner) => TokenActivityHelper {
                token_data_id: &binding,
                property_version: BigDecimal::zero(),
                from_address: Some(inner.buyer.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: Some(inner.coin_type.to_string()),
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazDelistEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.seller.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazListEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.seller.clone()),
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazSellEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.seller.clone()),
                to_address: Some(inner.buyer.clone()),
                token_amount: inner.amount.clone(),
                coin_type: Some(inner.coin_type.to_string()),
                coin_amount: Some(inner.price.clone()),
            },
            TokenEvent::TopazSendEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.sender.clone()),
                to_address: Some(inner.receiver.clone()),
                token_amount: inner.amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::Souffl3BuyTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.token_owner.clone()),
                to_address: Some(inner.buyer.clone()),
                token_amount: inner.token_amount.clone(),
                coin_type: None,
                coin_amount: Some(inner.coin_per_token.clone()),
            },
            TokenEvent::Souffl3CancelListTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: None,
                to_address: None,
                token_amount: inner.token_amount.clone(),
                coin_type: None,
                coin_amount: None,
            },
            TokenEvent::Souffl3ListTokenEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: Some(inner.token_owner.clone()),
                to_address: None,
                token_amount: inner.token_amount.clone(),
                coin_type: None,
                coin_amount: Some(inner.coin_per_token.clone()),
            },
            TokenEvent::Souffl3TokenListEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: None,
                to_address: None,
                token_amount: inner.amount.clone(),
                coin_type: Some(inner.coin_type_info.to_string()),
                coin_amount: Some(inner.min_price.clone()),
            },
            TokenEvent::Souffl3TokenSwapEvent(inner) => TokenActivityHelper {
                token_data_id: &inner.token_id.token_data_id,
                property_version: inner.token_id.property_version.clone(),
                from_address: None,
                to_address: Some(inner.token_buyer.clone()),
                token_amount: inner.token_amount.clone(),
                coin_type: Some(inner.coin_type_info.to_string()),
                coin_amount: Some(inner.coin_amount.clone()),
            }
        };
        // onlyadd to volume if event contains "buy" or "sell"
        if event_type.contains("Buy")
            || event_type.contains("Sell")
            || event_type.contains("Swap")
        {
            let collection_data_id_hash = token_data_id.get_collection_data_id_hash();
            let volume = token_activity_helper.coin_amount.clone().unwrap_or(BigDecimal::zero());
            Some(Self {
                collection_data_id_hash,
                volume,
                inserted_at: txn_timestamp,
                last_transaction_version: txn_version,
            })
        } else {
            None
        }
    }
}
