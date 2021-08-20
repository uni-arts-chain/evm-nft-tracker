# evm-nft-tracker

This is a tool for discovering EVM-based NFTs, currently supporting the ERC-721 and ERC-1155 standards.
The program contains two main parts, a library and some executables. The library is the common part and is used by the executables to discover NFTs events.

The ERC-721 and ERC-1155 contracts deployed on EVM-compatible virtual machines are easily supported. A new EVM-compatible chain can be supported by simply creating an executable.

## Rationale

The [ERC-721](https://eips.ethereum.org/EIPS/eip-721) and [ERC - 1155](https://eips.ethereum.org/EIPS/eip-1155) are Ethereum protocals and are also supported by other projects running on EVM-compatible virtual machines. As these two standards are widely used by NFTs issued on EVM, this project only focuses on NFTs of ERC-721 and ERC-1155.

This project discovers NFTs by listening to the transfer events of ERC-721 and ERC-1155 contracts. Why only listen to the transfer events? The first reason is that the transfer events are sufficient. The transfer events include all transferring, minting and burning. The second is because this project is part of the NFT browser, other events that are not transfer are not needed. (it may be necessary to consider adding the URI event of ERC-1155, which will issue a URI event when the URI is modified. ERC-721 does not have the similar event type).

### Events used

##### ERC-721

```
event Transfer(address indexed _from, address indexed _to, uint256 indexed _tokenId);
```

##### ERC - 1155

```
event TransferSingle(address indexed _operator, address indexed _from, address indexed _to, uint256 _id, uint256 _value);
event TransferBatch(address indexed _operator, address indexed _from, address indexed _to, uint256[] _ids, uint256[] _values);
```

When `_from` is a zero address, this event is a minting.  
When `_to` is a zero address, this event is a burning.  
When `_from` and `_to` are both non-zero addresses, this event is a normal transfer event.  

### Determine if it is an ERC-721 or ERC-1155 contract

It is not possible to determine the type of a contract by events alone, because events can be the same for different types of contracts. if two event definitions has the same name and parameter types, they can produce the same kind of events . So there are other ways to determine whether the event belongs to an ERC-721 or ERC-1155 contract.

This is achieved through [ERC-165](https://eips.ethereum.org/EIPS/eip-165).

Both ERC-721 and ERC-1155 protocals specify that NFTs need to implement the interface of ERC-165. So, we can determine whether a contract is ERC-721 or ERC - 1155 by the interface provided by ERC-165.

```
function supportsInterface(bytes4 interfaceID) external view returns (bool);
```

When you call this method, its return value will tell you the result.

The interfaceID for ERC - 721 is `0x80ac58cd`.

The interfaceID for ERC -1155 is `0xd9b67a26`.

### Consider only visual NFTs

Neither ERC-721 nor ERC-1155 require that NFTs be visual, so some non-visual NFTs may exist.

Visualization is achieved through optional metadata extension. This project will ignore these NFTs that do not implement the metadata extension. Whether NFTs implement the metadata extension is also determined by [ERC-165](https://eips.ethereum.org/EIPS/eip-165).

The interfaceID of the metadata extension for ERC -721 is `0x5b5e139f`.

The interfaceID of the metadata extension for ERC -1155 is `0x0e89341c`.

Why ignore non-visual NFTs?  This project is part of [The NFT Explorer](https://github.com/uni-arts-chain/uniscan), it will only focus the visual NFTs, so this project is only concerned with visual NFTs.

## Project Structure

#### libs/nft-events

This is the core library for discovering NFTs, used by the executables to track blockchains.

The library is divided into two parts, ERC-721 and ER -1155 parts. The reason for not abstracting into one is that there are some differences between the two, and for the sake of easy modification, they are directly considered separately, and in the future, if they become stable, they can be considered to be merged.

The ERC-721 part contains three files, which are :  

- `erc721.rs`: This is the entry file that provides the `track_erc721_events` method that the executables use to discover ERC-721 events.

- `erc721_evm.rs`: This is the core file that implements how to fetch the required ERC-721 events from the blockchain.

- `erc721_db.rs`: This is the local database support. The database here is used to cache the ERC-721 metadata. The database uses sqlite3's persistent storage, so the data will still exist after restart.

If the library is to be used, all that is needed is to implement an executable to call the `track_erc721_events`  and two callbacks with your own logic.

The ERC-1155 part is similar, includes `erc1155.rs`, `erc1155_evm.rs` and `erc1155_db.rs` 

#### livenets/* and testnets/*

The crates under the two directories are specific executables for different blockchains, each implementing a tracker for their blockchain. These executables mainly call `track_erc721_events` and `track_erc1155_events` of `nft-events`  lib to get the NFT events for their respective chains, and each has its own parameters.

The executable runs and keeps getting NFT events, then sends the NFT events and NFT metadata to the message queue of the [The NFT Explorer](https://github.com/uni-arts-chain/uniscan) (currently they are only printed into the stdout).

## Challenges

1. huge amount of historical data, which needs to be synchronized for a long time to get all the data
2. Stability of blockchain nodes, as it takes a long time to connect nodes to get data, so stable nodes are needed for access. Nowadays, public nodes are generally limited in one way or another.
