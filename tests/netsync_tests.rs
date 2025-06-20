use rtp_midi_netsync::error::NetsyncError;
use rtp_midi_netsync::midi::{MidiEvent, MmcCommand};
use rtp_midi_netsync::midi::{
    MMC_LOCATE_LENGTH, MMC_START_STOP_LENGTH, MTC_FULL_FRAME_LENGTH, MTC_QUARTER_FRAME_LENGTH,
};
use rtp_midi_netsync::netsync::{master_netsync_flow, slave_netsync_flow};

#[test]
fn test_master_flow_mmc_play() {
    let event = MidiEvent::Mmc(MmcCommand::Play);
    let result = master_netsync_flow(&event);
    assert!(result.is_ok());

    let payload = result.unwrap();
    assert!(!payload.is_empty());
    assert_eq!(payload[0] & 0x0F, MMC_START_STOP_LENGTH as u8);
}

#[test]
fn test_master_flow_mmc_stop() {
    let event = MidiEvent::Mmc(MmcCommand::Stop);
    let result = master_netsync_flow(&event);
    assert!(result.is_ok());

    let payload = result.unwrap();
    assert_eq!(payload[0] & 0x0F, MMC_START_STOP_LENGTH as u8);
}

#[test]
fn test_master_flow_mmc_locate() {
    let event = MidiEvent::Mmc(MmcCommand::Locate {
        hour: 1,
        minute: 30,
        second: 45,
        frame: 15,
        subframe: 0,
    });
    let result = master_netsync_flow(&event);
    assert!(result.is_ok());

    let payload = result.unwrap();
    assert_eq!(payload[0] & 0x0F, MMC_LOCATE_LENGTH as u8);
}

#[test]
fn test_master_flow_mtc_quarter() {
    let event = MidiEvent::MtcQuarter {
        msg_type: 0,
        value: 5,
    };
    let result = master_netsync_flow(&event);
    assert!(result.is_ok());

    let payload = result.unwrap();
    assert_eq!(payload[0] & 0x0F, MTC_QUARTER_FRAME_LENGTH as u8);
}

#[test]
fn test_master_flow_mtc_full() {
    let event = MidiEvent::MtcFull {
        hour: 1,
        minute: 30,
        second: 45,
        frame: 15,
    };
    let result = master_netsync_flow(&event);
    assert!(result.is_ok());

    let payload = result.unwrap();
    assert_eq!(payload[0] & 0x0F, MTC_FULL_FRAME_LENGTH as u8);
}

#[test]
fn test_master_flow_invalid_event() {
    let event = MidiEvent::Other(vec![0x90, 0x60, 0x7F]);
    let result = master_netsync_flow(&event);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), NetsyncError::InvalidMasterEvent);
}

#[test]
fn test_slave_flow_insufficient_data() {
    let short_buf = &[0x02];
    let result = slave_netsync_flow(short_buf);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), NetsyncError::InvalidSlaveEvent);
}

#[test]
fn test_slave_flow_empty_buffer() {
    let empty_buf = &[];
    let result = slave_netsync_flow(empty_buf);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), NetsyncError::InvalidSlaveEvent);
}

#[test]
fn test_roundtrip_mmc_play() {
    let original_event = MidiEvent::Mmc(MmcCommand::Play);

    let payload = master_netsync_flow(&original_event).unwrap();
    let reconstructed_event = slave_netsync_flow(&payload).unwrap();

    assert_eq!(original_event, reconstructed_event);
}

#[test]
fn test_roundtrip_mtc_quarter() {
    let original_event = MidiEvent::MtcQuarter {
        msg_type: 3,
        value: 7,
    };

    let payload = master_netsync_flow(&original_event).unwrap();
    let reconstructed_event = slave_netsync_flow(&payload).unwrap();

    assert_eq!(original_event, reconstructed_event);
}

#[test]
fn test_roundtrip_mmc_locate() {
    let original_event = MidiEvent::Mmc(MmcCommand::Locate {
        hour: 2,
        minute: 15,
        second: 30,
        frame: 10,
        subframe: 50, // Send 50, but expect 0 back
    });

    let payload = master_netsync_flow(&original_event).unwrap();
    let reconstructed_event = slave_netsync_flow(&payload).unwrap();

    // Expect the subframe to be normalized to 0
    let expected_event = MidiEvent::Mmc(MmcCommand::Locate {
        hour: 2,
        minute: 15,
        second: 30,
        frame: 10,
        subframe: 0, // Always 0 in the reconstructed event
    });

    assert_eq!(expected_event, reconstructed_event);
}
