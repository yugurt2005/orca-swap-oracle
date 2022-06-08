use {
	solana_client::rpc_client::RpcClient,
	solana_sdk::{
		pubkey::Pubkey
	},
	solana_program::program_pack::*,

	std::str,
	std::str::FromStr,

	spl_token::state::*,
	spl_token_swap::state::*,

	spl_token_swap::curve::*,
};

const ENDPOINT: &str = "https://solana-api.projectserum.com";
const EXCHANGE: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";


struct TokenSwap {
	address: Pubkey,

	token1: Account,
	token1_pubkey: Pubkey,
	token1_amount: u64,

	token2: Account,
	token2_pubkey: Pubkey,
	token2_amount: u64,

	fee_structure: fees::Fees
}

impl TokenSwap {
	fn calculate_fees(&self, value: u128) -> u128 {
		let value = value as f64;

		let mut result: f64 = 0.0;

		result += value / self.fee_structure.trade_fee_denominator as f64 * self.fee_structure.trade_fee_numerator as f64;
		result += value / self.fee_structure.owner_trade_fee_denominator as f64 * self.fee_structure.owner_trade_fee_numerator as f64;

		return result as u128;
	}

	pub fn get_quote(&self, amount: u128) -> u128 {
		let token1_count = self.token1_amount as u128;
		let token2_count = self.token2_amount as u128;
		let invariant = token1_count * token2_count;

		let amount_after_fees = amount - self.calculate_fees(amount);

		return token2_count - invariant / (amount + token1_count);
	}

	pub fn new(client: &RpcClient, address: Pubkey, content: solana_sdk::account::Account) -> TokenSwap {
		let account = SwapVersion::unpack(&content.data).unwrap();

		let token1_pubkey = *account.token_a_account();
		let token1_data = client.get_account_data(&token1_pubkey).unwrap();
		let token1_account = Account::unpack_from_slice(&token1_data).unwrap();
		let token1_amount = token1_account.amount;

		let token2_pubkey = *account.token_b_account();
		let token2_data = client.get_account_data(&token2_pubkey).unwrap();
		let token2_account = Account::unpack_from_slice(&token2_data).unwrap();
		let token2_amount = token2_account.amount;

		let fee_structure: fees::Fees = account.fees().clone();

		TokenSwap {
			address: address,

			token1: token1_account,
			token1_pubkey,
			token1_amount,

			token2: token2_account,
			token2_pubkey,
			token2_amount,

			fee_structure
		}
	}
}

impl std::fmt::Display for TokenSwap {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		let tvl = self.token1_amount + self.token2_amount;
		let cnt = (tvl as f64 * 0.01) as u128;
		write!(formatter,
			"
			--- Swap Address ---
			{:?}

			--- Token 1 ---\n
			Pubkey: {:?}\n
			Amount: {:?}\n

			--- Token 2 --\n
			Pubkey: {:?}\n
			Amount: {:?}\n

			--- Exchange Rate ---
			{} <-> {} = {}
			",
			self.address,
			self.token1_pubkey,
			self.token1_amount,
			self.token2_pubkey,
			self.token2_amount,
			cnt,
			self.get_quote(cnt),
			self.get_quote(cnt) as f64 / cnt as f64
		)
	}
}

struct TokenRegistry {
	token_swap_list: Vec<TokenSwap> 
}

impl TokenRegistry {
	pub fn lookup(token1_string: &str, token2_string: &str) {
		// TODO
	}

	pub fn new() -> TokenRegistry {
		let mut token_swap_list: Vec<TokenSwap> = vec![];

		let client: RpcClient = RpcClient::new(ENDPOINT);
		let liquidity_pools = client.get_program_accounts(
			&Pubkey::from_str(EXCHANGE).unwrap()
		).unwrap();

		println!("Number of Swap-Pairs Considered: {}", liquidity_pools.len());

		for (address, content) in liquidity_pools {
			let swap = TokenSwap::new(
				&client,
				address,
				content,
			);

			if swap.token1_amount == 0 || swap.token2_amount == 0 {
				continue;
			}

			println!("{}", swap);

			token_swap_list.push(swap);
		}

		TokenRegistry {
			token_swap_list: token_swap_list
		}
	}
}

pub fn main() {
	let interface = TokenRegistry::new();
}
