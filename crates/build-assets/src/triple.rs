use target_lexicon::{Architecture, Environment, OperatingSystem, Triple};

pub trait TripleExt {
    fn apt_toolchain(&self) -> Option<&'static str>;
    fn cc_compiler(&self) -> Option<&'static str>;
    fn is_windows(&self) -> bool;
}

impl TripleExt for Triple {
    fn apt_toolchain(&self) -> Option<&'static str> {
        Some(match self {
            Triple {
                architecture: Architecture::Aarch64(_),
                operating_system: OperatingSystem::Linux,
                ..
            } => "gcc-aarch64-linux-gnu",
            Triple {
                architecture: Architecture::Arm(_),
                operating_system: OperatingSystem::Linux,
                environment: Environment::Gnueabi | Environment::Musleabi,
                ..
            } => "gcc-arm-linux-gnueabi",
            Triple {
                architecture: Architecture::Arm(_),
                operating_system: OperatingSystem::Linux,
                environment: Environment::Gnueabihf | Environment::Musleabihf,
                ..
            } => "gcc-arm-linux-gnueabihf",
            Triple {
                architecture: Architecture::Arm(_),
                operating_system: OperatingSystem::None_,
                environment: Environment::Eabi,
                ..
            } => "gcc-arm-none-eabi",
            Triple {
                architecture: Architecture::X86_32(_) | Architecture::X86_64,
                operating_system: OperatingSystem::Linux,
                environment: Environment::Musl,
                ..
            } => "musl-tools",
            _ => return None,
        })
    }

    fn cc_compiler(&self) -> Option<&'static str> {
        Some(match self {
            Triple {
                architecture: Architecture::Aarch64(_),
                operating_system: OperatingSystem::Linux,
                ..
            } => "aarch64-linux-gnu-gcc",
            Triple {
                architecture: Architecture::Arm(_),
                operating_system: OperatingSystem::Linux,
                environment: Environment::Gnueabi | Environment::Musleabi,
                ..
            } => "arm-linux-gnueabi-gcc",
            Triple {
                architecture: Architecture::Arm(_),
                operating_system: OperatingSystem::Linux,
                environment: Environment::Gnueabihf | Environment::Musleabihf,
                ..
            } => "arm-linux-gnueabihf-gcc",
            Triple {
                architecture: Architecture::Arm(_),
                operating_system: OperatingSystem::None_,
                environment: Environment::Eabi,
                ..
            } => "arm-none-eabi-gcc",
            Triple {
                architecture: Architecture::X86_32(_) | Architecture::X86_64,
                operating_system: OperatingSystem::Linux,
                environment: Environment::Musl,
                ..
            } => "musl-gcc",
            _ => return None,
        })
    }

    fn is_windows(&self) -> bool {
        self.operating_system == OperatingSystem::Windows
    }
}
