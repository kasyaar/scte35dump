use std::cell;
use mpeg2ts_reader::demultiplex;
use mpeg2ts_reader::packet;
use mpeg2ts_reader::psi;
use mpeg2ts_reader::StreamType;
use std::collections::HashMap;
use hexdump;
use bitreader;
use std::fmt;
use scte35_reader;

pub struct DumpSpliceInfoProcessor;
impl scte35_reader::SpliceInfoProcessor for DumpSpliceInfoProcessor {
    fn process(&self, header: scte35_reader::SpliceInfoHeader, command: scte35_reader::SpliceCommand, descriptors: scte35_reader::SpliceDescriptorIter) {
        println!("{:?} {:#?}", header, command);
        for d in descriptors {
            println!("  {:?}", d);
        }
    }
}

struct Scte35StreamConsumer {
    section: psi::SectionPacketConsumer,
}
impl Default for Scte35StreamConsumer {
    fn default() -> Self {
        Scte35StreamConsumer {
            section: psi::SectionPacketConsumer::new(scte35_reader::Scte35SectionProcessor::new(DumpSpliceInfoProcessor))
        }
    }
}
impl Scte35StreamConsumer {
    fn construct(stream_info: &demultiplex::StreamInfo) -> Box<cell::RefCell<demultiplex::PacketFilter>> {
        // TODO: check for registration descriptor per SCTE-35, section 8.1
        for d in stream_info.descriptors() {
            match d {
                Ok(desc) => println!("scte35 descriptor tag={:?}", desc.tag()),
                Err(e) => println!("Problem reading descriptor: {:?}", e),
            }
        }
        let consumer = Scte35StreamConsumer::default();
        Box::new(cell::RefCell::new(consumer))
    }
}
impl packet::PacketConsumer<demultiplex::FilterChangeset> for Scte35StreamConsumer {
    fn consume(&mut self, pk: packet::Packet) -> Option<demultiplex::FilterChangeset> {
        self.section.consume(pk);
        None
    }
}

pub fn create_demux() -> demultiplex::Demultiplex {
    let mut table: HashMap<StreamType, fn(&demultiplex::StreamInfo)->Box<cell::RefCell<demultiplex::PacketFilter>>>
    = HashMap::new();

    table.insert(StreamType::Private(0x86), Scte35StreamConsumer::construct);
    let ctor = demultiplex::StreamConstructor::new(demultiplex::NullPacketFilter::construct, table);
    demultiplex::Demultiplex::new(ctor)
}