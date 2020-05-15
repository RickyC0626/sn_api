// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{
        get_from_arg_or_stdin, get_from_stdin, hex_to_xorname, parse_stdin_arg, serialise_output,
    },
    OutputFmt,
};
use safe_api::{xorurl::XorUrl, Safe};
use structopt::StructOpt;

// Default type tag to use for the Sequence
const DEFAULT_SEQUENCE_TYPE_TAG: u64 = 1_200;

#[derive(StructOpt, Debug)]
pub enum SeqSubCommands {
    #[structopt(name = "put")]
    /// Create a new Sequence on the SAFE Network
    Put {
        /// The data to store in the new Sequence as first element.  Specify '-' to read from stdin
        data: String,
        /// The type tag to be set
        #[structopt(long = "type")]
        type_tag: Option<u64>,
        /// The Xor name address (in hex) where to store the Sequence
        #[structopt(long = "xorname")]
        xorname: Option<String>,
    },
    #[structopt(name = "append")]
    /// Append an element to an existing Sequence on the network
    Append {
        /// The data to append to the Sequence
        #[structopt(parse(from_str = parse_stdin_arg))]
        data: String,
        /// The target Sequence to append the data to
        target: Option<String>,
    },
}

pub async fn seq_commander(
    cmd: SeqSubCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        SeqSubCommands::Put {
            data,
            type_tag,
            xorname,
        } => {
            let tag = type_tag.unwrap_or_else(|| DEFAULT_SEQUENCE_TYPE_TAG);
            let xorname = match xorname.as_ref() {
                Some(hex_str) => Some(hex_to_xorname(hex_str)?),
                None => None,
            };

            // If data is '-' then we read arg from STDIN
            let xorurl = if data == "-" {
                safe.sequence_create(
                    &get_from_stdin(Some("...awaiting data that will be stored from STDIN"))?,
                    xorname,
                    tag,
                )
                .await?
            } else {
                safe.sequence_create(data.as_bytes(), xorname, tag).await?
            };

            if OutputFmt::Pretty == output_fmt {
                println!("Sequence created at: \"{}\"", xorurl);
            } else {
                print_serialized_output(xorurl, output_fmt)?;
            }

            Ok(())
        }
        SeqSubCommands::Append { data, target } => {
            let target_url =
                get_from_arg_or_stdin(target, Some("...awaiting target URl from STDIN"))?;

            // If data is '-' then we read arg from STDIN
            let item = if data.is_empty() {
                get_from_stdin(Some("...awaiting data to append from STDIN"))?
            } else {
                data.as_bytes().to_vec()
            };

            // Append item to the Sequence on the Network
            safe.sequence_append(&target_url, &item).await?;

            if OutputFmt::Pretty == output_fmt {
                println!("Data appended to the Sequence: \"{}\"", target_url);
            } else {
                print_serialized_output(target_url, output_fmt)?;
            }

            Ok(())
        }
    }
}

fn print_serialized_output(xorurl: XorUrl, output_fmt: OutputFmt) -> Result<(), String> {
    /*let url = match XorUrlEncoder::from_url(&xorurl) {
        Ok(mut xorurl_encoder) => {
            xorurl_encoder.set_content_version(Some(version));
            xorurl_encoder.to_string()
        }
        Err(_) => xorurl,
    };*/
    println!("{}", serialise_output(&xorurl, output_fmt));

    Ok(())
}
