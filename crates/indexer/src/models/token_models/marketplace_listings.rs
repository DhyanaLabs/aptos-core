// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use std::collections::HashMap;

use super::token_utils::{TokenDataIdType, TokenEvent};
use crate::{
    schema::{current_marketplace_listings},
    util::{parse_timestamp},
};
use aptos_api_types::{Event as APIEvent, Transaction as APITransaction};
use bigdecimal::{BigDecimal, Zero};
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Serialize)]
#[diesel(primary_key(
    market_address,
    token_data_id_hash
))]
#[diesel(table_name = current_marketplace_listings)]
pub struct CurrentMarketplaceListing {
    pub collection_data_id_hash: String,
    pub market_address: String,
    pub token_data_id_hash: String,
    pub property_version: BigDecimal,
    pub creator_address: String,
    pub collection_name: String,
    pub name: String,
    pub seller: String,
    pub amount: BigDecimal,
    pub price: BigDecimal,
    pub event_type: String,
    pub inserted_at: chrono::NaiveDateTime,
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


impl CurrentMarketplaceListing {
    pub fn from_transaction(transaction: &APITransaction) -> HashMap<String, Self> {
        let mut current_marketplace_listings: HashMap<String, Self> = HashMap::new();
        if let APITransaction::UserTransaction(user_txn) = transaction {
            for event in &user_txn.events {
                let txn_version = user_txn.info.version.0 as i64;
                let event_type = event.typ.to_string();
                match TokenEvent::from_event(event_type.as_str(), &event.data, txn_version).unwrap()
                {
                    Some(token_event) => {
                        let parsed_event = Self::from_parsed_event(
                            &event_type,
                            event,
                            &token_event,
                            txn_version,
                            parse_timestamp(user_txn.timestamp.0, txn_version),
                        );
                    if let Some(current_marketplace_listing) =  parsed_event {
                        current_marketplace_listings.insert(
                            current_marketplace_listing.token_data_id_hash.clone(), 
                            current_marketplace_listing.into()
                        )
                        } else {
                            None
                        }
                    }
                    None => None
                };
            }
        }
        current_marketplace_listings
    }

    pub fn from_parsed_event(
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
        };
        // only update listing info if event type contains "list", "delist", "buy", "sell", 'change', 'send', or 'claim', else return None
        if event_type.contains("List")
            || event_type.contains("Delist")
            || event_type.contains("Buy")
            || event_type.contains("Sell")
            || event_type.contains("Change")
            || event_type.contains("CancelList")
            || event_type.contains("Fill")
            || event_type.contains("Send")
            || event_type.contains("Auction")
        {
            // market address is "0xd1fd99c1944b84d1670a2536417e997864ad12303d19eac725891691b04d614e" for blue/bluemove, "0x2c7bccf7b31baf770fdbcc768d9e9cb3d87805e255355df5db32ac9a669010a2" for topaz, and "0xf6994988bd40261af9431cd6dd3fcf765569719e66322c7a05cc78a89cd366d4" for souffl3
            let mut market_address = event_type.split("::").next().unwrap(); //
            if !(event_type.contains("List") || event_type.contains("Auction")) || event_type.contains("CancelList") || event_type.contains("Delist") {
                market_address = "";
            } 
            let token_data_id_hash = token_data_id.to_hash();
            let creator_address = token_data_id.creator.clone();
            let collection_name = token_data_id.collection.clone();
            let name = token_data_id.name.clone();
            let seller = token_activity_helper.from_address.clone().unwrap_or("".to_owned());
            let amount = token_activity_helper.token_amount.clone();
            let price = token_activity_helper.coin_amount.clone().unwrap_or(BigDecimal::zero());
            Some(Self {
                collection_data_id_hash: token_data_id.get_collection_data_id_hash(),
                market_address: market_address.to_owned(),
                token_data_id_hash,
                property_version: token_activity_helper.property_version.clone(),
                creator_address,
                collection_name,
                name,
                seller,
                amount,
                price,
                event_type: event_type.to_owned(),
                inserted_at: txn_timestamp,
            })
        } else {
            None
        }
    }
}