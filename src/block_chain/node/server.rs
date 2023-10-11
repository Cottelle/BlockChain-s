use std::{
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};

use tracing::info;

use crate::block_chain::{
    block::{self, mine, Block},
    blockchain::Blockchain,
    // shared::Shared,
    node::network::{Network, Packet, TypeBlock},
    transaction::Transaction,
};
use crate::friendly_name::*;

pub enum RequestNetwork {
    SendHash((u64, SocketAddr)),
    NewBlock(Block),
}

pub enum RequestServer {
    AnswerHash((Block, SocketAddr)),
    AskHash(u64),
}

pub enum NewBlock {
    Mined(Block),
    Network(Block),
}

pub enum Event {
    NewBlock(NewBlock),
    HashReq((u64, SocketAddr)),
    Transaction(Transaction),
}

pub struct Server {
    name: String,
    network: Network, // blockchaine
    //miner
    id: u64,
    blockchain: Blockchain,
}

impl Server {
    pub fn new(network: Network) -> Self {
        let name =
            get_friendly_name(network.get_socket()).expect("generation name from ip imposble");
        let id = get_fake_id(&name);
        Self {
            name,
            network,
            id: id,
            blockchain: Blockchain::new(),
        }
    }
    pub fn start(mut self) {
        println!(
            "Server started {} facke id {} -> {:?}",
            &self.name,
            get_fake_id(&self.name),
            self.network
        );
        info!("server mode");
        let id = get_fake_id(&self.name);

        //
        // need to link new stack of transaction because the miner need continue to mine without aprouvale of the network

        let event_channel = mpsc::channel::<Event>();

        //transaction
        thread::spawn(move || {
            /*
            Mutex of
            when receive a transaction
             */
        });

        self.network.clone().start(
            // mined_block_rx,
            event_channel.0.clone(),
            // server_network_rx,
        );

        // println!("blockaine recus{:?}", blockaine);

        self.server_runtime(
            self.id,
            event_channel, // server_network_tx,
        );
    }

    fn server_runtime(
        &mut self,
        finder: u64,
        // mined_block_tx: Sender<Block>,
        event_channels: (Sender<Event>, Receiver<Event>),
        // server_network_tx: Sender<RequestServer>,
    ) {
        info!("Runtime server start");

        let actual_top_block = Arc::new(Mutex::new(self.blockchain.last_block()));
        let actual_top_block_cpy = actual_top_block.clone();

        thread::Builder::new()
            .name("Miner".to_string())
            .spawn(move || {
                info!("start Miner");
                mine(finder, &actual_top_block_cpy, event_channels.0);
            })
            .unwrap();

        loop {
            // let new_block: Block = match new_block_rx.recv().unwrap(){
            //     BlockFrom::Mined(block) => {
            //         //network send
            //         block
            // }
            //     BlockFrom::Network(block) => block,
            // };
            match event_channels.1.recv().unwrap() {
                Event::HashReq((hash, dest)) => {
                    if let Some(block) = self.blockchain.get_block(hash) {
                        self.network
                            .send_packet(&Packet::Block(TypeBlock::Block(block.clone())), &dest)
                    }
                }
                Event::NewBlock(new_block) => {
                    let new_block = match new_block {
                        NewBlock::Mined(b) => {
                            self.network
                                .broadcast(Packet::Block(TypeBlock::Block(b.clone())));
                            b
                        }
                        NewBlock::Network(b) => b,
                    };
                    println!("New block");
                    let (new_top_block, block_need) = self.blockchain.append(&new_block);

                    if let Some(top_block) = new_top_block {
                        let mut lock_actual_top_block = actual_top_block.lock().unwrap();
                        *lock_actual_top_block = top_block;
                    }

                    if let Some(needed_block) = block_need {
                        self.network
                            .broadcast(Packet::Block(TypeBlock::Hash(needed_block)));
                    }
                }
                Event::Transaction(_) => todo!(),
            }
        }
    }
}
