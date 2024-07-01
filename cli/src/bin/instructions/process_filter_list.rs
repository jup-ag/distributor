use jito_merkle_tree::tree_node::ui_amount_to_token_amount;

use crate::*;

/// filter from pubkey only
pub fn process_filter_list(filter_list_args: &FilterListArgs) {
    let community_list = CsvEntry::new_from_file(&filter_list_args.csv_path).unwrap();
    // let test_list: Vec<String> = get_pre_list();
    let mut full_list = vec![];
    for node in community_list.iter() {
        let addr = Pubkey::from_str(&node.pubkey);
        if addr.is_err() {
            println!("{} is not pubkey", node.pubkey);
            continue;
        }
        full_list.push((addr.unwrap(), filter_list_args.amount));
    }

    // for node in test_list.iter() {
    //     let addr = Pubkey::from_str(&node);
    //     if addr.is_err() {
    //         println!("{} is not pubkey", node);
    //         continue;
    //     }
    //     full_list.push((addr.unwrap(), filter_list_args.amount));
    // }

    // remove duplicate
    println!("num node {} ", full_list.len());
    full_list.sort_unstable();
    full_list.dedup_by(|a, b| a.0 == b.0);
    println!("num node {} ", full_list.len());

    let mut wtr = Writer::from_path(&filter_list_args.destination_path).unwrap();
    wtr.write_record(&["pubkey", "amount"]).unwrap();
    for address in full_list.iter() {
        wtr.write_record(&[address.0.to_string(), address.1.to_string()])
            .unwrap();
    }

    wtr.flush().unwrap();
}

/// filter from pubkey only
pub fn process_filter_list_fixed(filter_list_args: &FilterListFixedArgs) {
    let community_list = CsvEntry::new_from_file(&filter_list_args.csv_path).unwrap();
    // let test_list: Vec<String> = get_pre_list();
    let mut full_list = vec![];
    let mut total_amount = 0u64;
    for node in community_list.iter() {
        let addr = Pubkey::from_str(&node.pubkey);
        if addr.is_err() {
            println!("{} is not pubkey", node.pubkey);
            continue;
        }
        full_list.push((addr.unwrap(), node.amount.clone()));
        total_amount = total_amount
            .checked_add(ui_amount_to_token_amount(&node.amount, 0))
            .unwrap();
    }

    // remove duplicate
    println!("num node before sort {} ", full_list.len());
    full_list.sort_unstable();
    full_list.dedup_by(|a, b| a.0 == b.0);
    println!("num node after sort {} ", full_list.len());

    println!("total amount {}", total_amount);

    let mut wtr = Writer::from_path(&filter_list_args.destination_path).unwrap();
    wtr.write_record(&["pubkey", "amount"]).unwrap();
    for address in full_list.iter() {
        wtr.write_record(&[address.0.to_string(), address.1.to_string()])
            .unwrap();
    }

    wtr.flush().unwrap();
}
