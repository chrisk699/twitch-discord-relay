use regex::Regex;

pub struct IrcParser {
	regex: Regex
}

pub struct IrcMessage {
	pub raw: String,
	pub prefix: String,
	pub nick: String,
	pub username: String,
	pub hostname: String,
	pub command: String,
	pub params: Vec<String>
}

impl IrcParser {


	pub fn new() -> IrcParser {
			
		IrcParser {
			/*
				Regex that captures a reply received from an irc server
				Source: https://gist.github.com/datagrok/380449c30fd0c5cf2f30
			
				Explanation:

				  ^ # We'll match the whole line. Start.
				    # Optional prefix and the space that separates it
				    # from the next thing. Prefix can be a servername,
				    # or nick[[!user]@host]
				  (?::(          # This whole set is optional but if it's
				                 # here it begins with : and ends with space
				    ([^@!\ ]*)   # nick
				    (?:          # then, optionally user/host
				      (?:        # but user is optional if host is given
				        !([^@]*) # !user
				      )?         # (user was optional)
				      @([^\ ]*)  # @host
				    )?           # (host was optional)
				  )\ )?          # ":nick!user@host " ends
				  ([^\ ]+)       # IRC command (required)
				  # Optional args, max 15, space separated. Last arg is
				  # the only one that may contain inner spaces. More than
				  # 15 words means remainder of words are part of 15th arg.
				  # Last arg may be indicated by a colon prefix instead.
				  # Pull the leading and last args out separately; we have
				  # to split the former on spaces.
				  (
				    (?:
				      \ [^:\ ][^\ ]* # space, no colon, non-space characters
				    ){0,14}          # repeated up to 14 times
				  )                  # captured in one reference
				  (?:\ :?(.*))?      # the rest, does not capture colon.
				  $ # EOL
			
			*/
			regex: Regex::new(r#"^(?::(([^@!\s]*)(?:(?:!([^@]*))?@([^\s]*))?)\s)?([^\s]+)((?:\s[^:\s][^\s]*){0,14})(?:\s:?(.*))?$"#).unwrap()
		}
	}

	pub fn parse(&self, str: String) -> IrcMessage {
		let caps = self.regex.captures(&str).unwrap();

		// Extract variable parameters from capture group 7 and counting
		let params: Vec<String> = caps.iter().skip(6).map(|e| {
			match e {
				Some(val) => val.as_str().to_string(),
				None => "".to_string()
			}
			
		}).filter(|s| !s.is_empty()).collect();

		IrcMessage {
			raw: caps.get(0).map_or("".to_string(), |m| m.as_str().to_string()),
			prefix: caps.get(1).map_or("".to_string(), |m| m.as_str().to_string()),
			nick: caps.get(2).map_or("".to_string(), |m| m.as_str().to_string()),
			username: caps.get(3).map_or("".to_string(), |m| m.as_str().to_string()),
			hostname: caps.get(4).map_or("".to_string(), |m| m.as_str().to_string()),
			command: caps.get(5).map_or("".to_string(), |m| m.as_str().to_string()),
			params: params
		}
	}
}
