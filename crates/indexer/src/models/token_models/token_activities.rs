// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use super::token_utils::{TokenDataIdType, TokenEvent};
use crate::{
    schema::token_activities,
    util::{parse_timestamp},
};
use aptos_api_types::{Event as APIEvent, Transaction as APITransaction};
use bigdecimal::{BigDecimal, Zero};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(
    transaction_version,
    event_account_address,
    event_creation_number,
    event_sequence_number
))]
#[diesel(table_name = token_activities)]
pub struct TokenActivity {
    pub transaction_version: i64,
    pub event_account_address: String,
    pub event_creation_number: i64,
    pub event_sequence_number: i64,
    pub token_data_id_hash: String,
    pub property_version: BigDecimal,
    pub creator_address: String,
    pub collection_name: String,
    pub name: String,
    pub transfer_type: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_amount: BigDecimal,
    pub coin_type: Option<String>,
    pub coin_amount: Option<BigDecimal>,
    pub collection_data_id_hash: String,
    pub transaction_timestamp: chrono::NaiveDateTime,
}

/// A simplified TokenActivity (excluded common fields) to reduce code duplication
struct TokenActivityHelper<'a> {
    pub token_data_id: &'a TokenDataIdType,
    pub property_version: BigDecimal,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_amount: BigDecimal,
    pub coin_type: Option<String>,
    pub coin_amount: Option<BigDecimal>,
}

impl TokenActivity {
    pub fn from_transaction(transaction: &APITransaction) -> Vec<Self> {
        let mut token_activities = vec![];
        if let APITransaction::UserTransaction(user_txn) = transaction {
            for event in &user_txn.events {
                let txn_version = user_txn.info.version.0 as i64;
                let event_type = event.typ.to_string();
                match TokenEvent::from_event(event_type.as_str(), &event.data, txn_version).unwrap()
                {
                    Some(token_event) => token_activities.push(Self::from_parsed_event(
                        &event_type,
                        event,
                        &token_event,
                        txn_version,
                        parse_timestamp(user_txn.timestamp.0, txn_version),
                    )),
                    None => {}
                };
            }
        }
        token_activities
    }

    pub fn from_parsed_event(
        event_type: &str,
        event: &APIEvent,
        token_event: &TokenEvent,
        txn_version: i64,
        txn_timestamp: chrono::NaiveDateTime,
    ) -> Self {
        let event_account_address = &event.guid.account_address.to_string();
        let event_creation_number = event.guid.creation_number.0 as i64;
        let event_sequence_number = event.sequence_number.0 as i64;
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
        let token_data_id = token_activity_helper.token_data_id;
        Self {
            event_account_address: event_account_address.to_string(),
            event_creation_number,
            event_sequence_number,
            token_data_id_hash: token_data_id.to_hash(),
            property_version: token_activity_helper.property_version,
            collection_data_id_hash: token_data_id.get_collection_data_id_hash(),
            creator_address: token_data_id.get_creator_address(),
            collection_name: token_data_id.get_collection_trunc(),
            name: token_data_id.get_name_trunc(),
            transaction_version: txn_version,
            transfer_type: event_type.to_string(),
            from_address: token_activity_helper.from_address,
            to_address: token_activity_helper.to_address,
            token_amount: token_activity_helper.token_amount,
            coin_type: token_activity_helper.coin_type,
            coin_amount: token_activity_helper.coin_amount,
            transaction_timestamp: txn_timestamp,
        }
    }
}
