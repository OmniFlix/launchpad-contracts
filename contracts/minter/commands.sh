Start  sudo omniflixhubd start --home /Users/adnandenizcorlu/.test 

Store omniflixhubd tx wasm store omniflix_minter.wasm --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3

Instantiate omniflixhubd tx wasm instantiate 8 '{}' --label test --no-admin --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3

create   omniflixhubd tx wasm execute omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474 '{"CreateDenom":{"id":"onftdenomvalid3","creation_fee":"100000000","description":"test","name":"test","symbol":"test2","preview_uri":"test","schema":"test","sender":"omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474"}}' --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3 --amount 100000000uflix
    
mint    omniflixhubd tx wasm execute omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474 '{"MintDenom":{"id":"onftdenomvalid3","denom_id":"onftdenomvalid3","data":"test","transferable":true , "extensible":true,"nsfw":false,"royalty_share":"0.1","sender":"omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474","recipient":"omniflix1pvrwmjuusn9wh34j7y520g8gumuy9xtl3gvprlljfdpwju3x7ucsu4gssy"}}' --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3 --amount 100000000uflix

burn    omniflixhubd tx wasm execute omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474 '{"BurnDenom":{"id":"onft9702039419d14d81b6a69828d92d54cd","denom_id":"onftdenombfab455b89594a2abbbe245075e6d957","sender":"omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474"}}' --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3 --amount 100000000uflix

Test omniflixhubd tx wasm execute omniflix1eyfccmjm6732k7wp4p6gdjwhxjwsvje44j0hfx8nkgrm8fs7vqfsfnzrge '{"test":{"id":"onftdenomvalid2","creation_fee":{"amount":"100","denom":"uflix"},"description":"test","name":"test","symbol":"test","preview_uri":"test","schema":"test","sender":"omniflix1eyfccmjm6732k7wp4p6gdjwhxjwsvje44j0hfx8nkgrm8fs7vqfsfnzrge" , "type_url":"/OmniFlix.onft.v1beta1.MsgTransferONFT"}}' --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3 --amount 100000000uflix

- New Minter Commands
Optimize sudo docker run --rm -v "$(pwd)":/code   --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target   --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry   --platform linux/amd64   cosmwasm/rust-optimizer:0.12.13

Start  sudo omniflixhubd start --home /Users/adnandenizcorlu/.test 

Store omniflixhubd tx wasm store omniflix_minter.wasm --from test --chain-id test-1 --fees 1000000uflix --gas auto --gas-adjustment 1.3

Instantiate omniflixhubd tx wasm instantiate 13 '{"collection_details": {"name": "test","description": "test","preview_uri": "test","schema": "test","symbol": "test3","id": "test3","extensible": true,"nsfw": false,"num_tokens": 100,"base_uri": "test"},"mint_price":"1000000","mint_denom": "uflix","start_time": "1698668574000000000","per_address_limit": 100,"royalty_ratio": 10}' --label test --no-admin --from test --chain-id test-1 --fees 1000000uflix --gas auto --amount 100000000uflix --gas-adjustment 1.3

Mint omniflixhubd tx wasm execute omniflix1xhcxq4fvxth2hn3msmkpftkfpw73um7s4et3lh4r8cfmumk3qsmssafzs4 '{"mint":{}}' --from test --chain-id test-1 --fees 1000000uflix --gas auto --amount 1000000uflix --gas-adjustment 1.3


Code_id: 8
Address: omniflix1vguuxez2h5ekltfj9gjd62fs5k4rl2zy5hfrncasykzw08rezpfsrwz474

Code_id: 9
Address: omniflix1vhndln95yd7rngslzvf6sax6axcshkxqpmpr886ntelh28p9ghuq676vhk

Code_id:12
Address: omniflix1657pee2jhf4jk8pq6yq64e758ngvum45gl866knmjkd83w6jgn3swl2z8l

Code_id: 13
Address: omniflix1xhcxq4fvxth2hn3msmkpftkfpw73um7s4et3lh4r8cfmumk3qsmssafzs4

Code_id: 14
Address: omniflix1wr6vc3g4caz9aclgjacxewr0pjlre9wl2uhq73rp8mawwmqaczsq656r5u

Code id: 15
Address: omniflix1dkcsehtk7vq2ta9x4kdazlcpr4s58xfxt3dvuj98025rmleg4g2qdhf3gk