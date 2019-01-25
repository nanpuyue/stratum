use super::*;

#[test]
fn get_subwork() {
    let work = r#"[
    "1",
    "320a79ca2b659f1a8b8119bb547f4ce4f56e0b0b0024c6070000000000000000",
    "02000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4b038c72080405ff115c622f4254432e434f4d2ffabe6d6d92c1c0f08fef10653fa93199f85160e9788b231135b667f61e85b4388e2b46950100000000000000",
    "ffffffff037493f24a0000000016001497cfc76442fe717f2a3f0cc9c175f7561b6619970000000000000000266a24aa21a9eda49de21a30362cd593a8c96f074502604725a74e4d82c1698c02314aea56dff500000000000000002952534b424c4f434b3a055f8e0dfe632de10a7b209acbef8373f07d888e6023596681c4bf881cdbb2f400000000",
    [
        "fd028a98cc947228779da1bae325548b028edc627cc478059078df2c7d31a665",
        "1b61abc0fb5aa7db6b7090863146da2700fac0be00fe8cfc7d3f039f93f88785",
        "f150f03d4738f35f3b44d3d2c0a352f96e1cfa9e4ad4ad3aaaee25186a94f633",
        "7834189b464a0b8d0c366a2fa9f7d293c9a351ba77a49c42bab33dbc739ef7ff",
        "20608a6670cf796c65ce9315e72214a2daab977228c9429e4b678c66912b2fa9",
        "0687644ae2d84fedc308b622c5e4aa02f176bc7fb44cbb0028256660c75dda9b",
        "a4b199d4ebe403cbc0fe1cbc9e2e47f58f1b411055d2e66ce33d89a1f9d05085",
        "0a9063ebd23e53ee19a45e5a385a6d176ce341e5958ddb30a3f61fd33ad89bcb",
        "cf523937c5f0e1a9afa2eda1eb8f303cc0e30aef54c19e5b17d779a33e40a6f7",
        "4efde9a74d29d924594294c71ac5c61d1d128cca9504526530517b3c1a795390",
        "15c3c798e192754ab0cd66acb2c0462dbf4d323ad9c5af4b4f40c17bb88984f7",
        "9a6774410f0d4df5aa881b88572b91af8b5bc8b46681a5570b7db9196057b7d3"
    ],
    "20000000",
    "1731d97c",
    "5c11ff05",
    false
]"#;
    let work: Work = serde_json::from_str(work).unwrap();
    let xnonce = (&Bytes::from("69bf584a".from_hex().unwrap()), Bytes::from("12345678".from_hex().unwrap()));
    let block_header = Bytes::from("20000000320a79ca2b659f1a8b8119bb547f4ce4f56e0b0b0024c6070000000000000000c7216feff133aab3a5414472e077a3735ca9839c15425536b8ad383bc099f99d5c11ff051731d97c".from_hex().unwrap());
    let midstate = Bytes::from("bf9213db167c49769ebbf9fa75c8fda449dfb01b75ce7a9c850b0d932b028d81".from_hex().unwrap());

    let subwork = work.subwork(xnonce.clone());
    assert_eq!(block_header, &subwork.block_header);
    assert_eq!(midstate, &subwork.midstate);

    let chunk1_itor = Chunk1Itor::new(&work, &xnonce,0x1fffe000u32);
    for chunk1 in chunk1_itor {
        println!("{:?}", chunk1);
    }
}
