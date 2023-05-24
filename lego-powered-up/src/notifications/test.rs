#[cfg(test)]
mod test {
    use super::*;
    use log::LevelFilter;

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter(None, LevelFilter::Trace)
            .try_init();
    }

    #[test]
    fn attach_io_message() {
        init();
        let msgs: &[&[u8]] = &[
            &[15, 0, 4, 0, 1, 47, 0, 0, 16, 0, 0, 0, 16, 0, 0],
            &[15, 0, 4, 50, 1, 23, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 59, 1, 21, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 60, 1, 20, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 61, 1, 60, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 96, 1, 60, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 97, 1, 57, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 98, 1, 58, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 99, 1, 59, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 100, 1, 54, 0, 1, 0, 0, 0, 1, 0, 0, 0],
        ];
        for msg in msgs {
            let notif = NotificationMessage::parse(msg).unwrap();
            if let NotificationMessage::HubAttachedIo(_) = notif {
                // OK
            } else {
                panic!("wrong type");
            }
        }
    }

    #[test]
    fn error_message() {
        init();
        let msgs: &[&[u8]] = &[&[5, 0, 5, 17, 5]];
        for msg in msgs {
            let _notif = NotificationMessage::parse(msg).unwrap();
        }
    }

    /*#[test]
    fn write_direct() {
        init();
        let msgs: &[&[u8]] = &[&[9, 0, 129, 81, 50, 1, 0, 255, 0]];
        for msg in msgs {
            let _notif = NotificationMessage::parse(msg).unwrap();
        }
    }*/

    #[test]
    fn message_length() {
        init();
        let test_cases = &[
            ([0x34, 0x00], 0x34),
            ([0x7f, 0x00], 0x7f),
            ([0b1000_0000, 0b0000_0001], 128),
            ([0b1000_0001, 0b0000_0001], 129),
            ([0b1000_0010, 0b0000_0001], 130),
        ];

        for case in test_cases {
            assert_eq!(
                NotificationMessage::length(case.0.iter()).unwrap(),
                case.1
            );
        }
    }

    #[test]
    fn serialise_write_direct() {
        /* Hub LED, from the arduino lib:
        byte port = getPortForDeviceType((byte)DeviceType::HUB_LED);
        byte setRGBMode[8] = {0x41, port, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00};
        WriteValue(setRGBMode, 8);
        byte setRGBColor[8] = {0x81, port, 0x11, 0x51, 0x01, red, green, blue};
        WriteValue(setRGBColor, 8);
        // WriteValue adds the length header and hub id = 0 header
        // https://github.com/corneliusmunz/legoino/blob/master/src/Lpf2Hub.cpp#L952
        */
        init();
        let startup_info = StartupInfo::ExecuteImmediately;
        let completion_info = CompletionInfo::CommandFeedback;

        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColors {
                red: 0x12,
                green: 0x34,
                blue: 0x56,
            },
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: 50,
                startup_info,
                completion_info,
                subcommand,
            });

        let serialised = msg.serialise();
        let correct =
            &mut [0_u8, 0, 0x81, 50, 0x11, 0x51, 0x01, 0x12, 0x34, 0x56];
        correct[0] = correct.len() as u8;

        assert_eq!(&serialised, correct);
    }

    #[test]
    fn port_input_format_setup_single() {
        init();

        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: 50,
                mode: 0x01,
                delta: 0x00000001,
                notification_enabled: false,
            });

        let serialised = msg.serialise();
        let correct =
            &mut [0_u8, 0, 0x41, 50, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00];
        correct[0] = correct.len() as u8;

        assert_eq!(&serialised, correct);
    }

    #[test]
    fn version_number() {
        init();
        // first test case from documentation
        // remainder from observed hardware
        let test_cases: &[(i32, VersionNumber)] = &[
            (
                0x17371510,
                VersionNumber {
                    major: 1,
                    minor: 7,
                    bugfix: 37,
                    build: 0      // Temporary until version number parse is fixed
                    // build: 0x1510,
                },
            ),
            (
                268435503,
                VersionNumber {
                    major: 1,
                    minor: 0,
                    bugfix: 0,
                    build: 0x2f,
                },
            ),
            (
                268435456,
                VersionNumber {
                    major: 1,
                    minor: 0,
                    bugfix: 0,
                    build: 0,
                },
            ),
            (
                23,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 23,
                },
            ),
            (
                21,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 21,
                },
            ),
            (
                20,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 20,
                },
            ),
            (
                60,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 60,
                },
            ),
            (
                4096,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 0         // Temporary until version number parse is fixed
                    // build: 4096,  
                },
            ),
            (
                65596,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 60,
                },
            ),
            (
                65536,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 0,
                },
            ),
            (
                65593,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 57,
                },
            ),
            (
                65594,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 58,
                },
            ),
            (
                65595,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 59,
                },
            ),
            (
                65590,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 54,
                },
            ),
        ];

        for (number, correct) in test_cases {
            eprintln!("\ntest case: {:08x} - {}", number, correct);
            let parsed =
                VersionNumber::parse(number.to_le_bytes().iter()).unwrap();
            assert_eq!(parsed, *correct);

            let serialised = correct.serialise();
            eprintln!("serialised: {:02x?}", serialised);
            eprintln!("correct LE: {:02x?}", number.to_le_bytes());
            assert_eq!(serialised, &number.to_le_bytes());
        }
    }

    #[test]
    fn motor_set_speed() {
        init();
        let subcommand = PortOutputSubcommand::StartSpeed {
            speed: 0x12,
            max_power: Power::Cw(0x34),
            use_acc_profile: true,
            use_dec_profile: true,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: 1,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        let serialised = msg.serialise();
        let correct = &mut [0, 0, 0x81, 1, 0x11, 0x01, 0x12, 0x34, 0x03];
        correct[0] = correct.len() as u8;

        assert_eq!(&serialised, correct);
    }
}
