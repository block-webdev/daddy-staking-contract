use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub total_nft_count: u64   // 8
}

#[account]
#[derive(Default)]
pub struct UserPool {
    // 380
    pub owner: Pubkey,                       // 32
    pub rand : Pubkey,
    pub item_count: u8,                     // 1
    pub nft_mint_list: [Pubkey; NFT_STAKE_MAX_COUNT], // 32 * 10 = 320
    pub rarity_list: [u8; NFT_STAKE_MAX_COUNT], // 1 * 10 = 10
    pub reward_time: i64,                    // 8
    pub stake_mode: u8, // 1, value = 0 : active, 1 : passive - 7 days, 2 : passive - 30 days
    pub stake_time: i64,       // 8
}
// impl Default for UserPool {
//     #[inline]
//     fn default() -> UserPool {
//         UserPool {
//             owner: Pubkey::default(),
//             item_count: 0,
//             items: [
//                 StakedNFT {
//                     ..Default::default()
//                 }; NFT_STAKE_MAX_COUNT
//             ],
//             reward_time: 0,
//             stake_mode: 0,
//             stake_time: 0,
//         }
//     }
// }

impl UserPool {
    pub fn add_nft(&mut self, nft_mint: Pubkey, rarity: u8) -> Result<u8> {
        require!(self.item_count < NFT_STAKE_MAX_COUNT as u8, StakingError::IndexOverflow);

        self.nft_mint_list[self.item_count as usize] = nft_mint;
        self.rarity_list[self.item_count as usize] = rarity;
        self.item_count += 1;

        Ok(self.item_count)
    }

    pub fn remove_nft(&mut self, owner: Pubkey, nft_mint: Pubkey) -> Result<u8> {
        require!(self.owner.eq(&owner), StakingError::InvalidOwner);
        let mut withdrawn: u8 = 0;
        for i in 0..self.item_count {
            let index = i as usize;
            if self.nft_mint_list[index].eq(&nft_mint) {
                // remove nft
                if i != self.item_count - 1 {
                    let last_idx = self.item_count - 1;
                    self.nft_mint_list[index] = self.nft_mint_list[last_idx as usize];
                    self.rarity_list[index] = self.rarity_list[last_idx as usize];
                }
                self.item_count -= 1;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);

        Ok(self.item_count)
    }

    pub fn calc_reward(&mut self, now: i64) -> Result<u64> {
        let mut total_reward: u64 = 0;
        if self.stake_mode == 0 {
            total_reward = 3;
        } else if self.stake_mode == 1 {
            total_reward = 5;
            if self.item_count != 0 {
                total_reward += self.rarity_list[0] as u64;
            }
        } else if self.stake_mode == 2 {
            total_reward = 7;
            if self.item_count != 0 {
                total_reward += self.rarity_list[0] as u64;
            }
        }

        for i in 1..self.item_count {
            if self.stake_mode == 0 {
                total_reward += 2;
            } else if self.stake_mode == 1 {
                total_reward += 4 + self.rarity_list[i as usize] as u64;
            } else {
                total_reward += 6 + self.rarity_list[i as usize] as u64;
            }
        }

        self.reward_time = now;
        Ok(total_reward)
    }
}

