// Copyright (c) 2018 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

use hcore;
use hcore::package::{self, Identifiable};

pub use super::generated::types::*;
use manager;

impl fmt::Display for ApplicationEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.get_application(), self.get_environment())
    }
}

impl fmt::Display for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.has_version() && self.has_release() {
            write!(
                f,
                "{}/{}/{}/{}",
                self.get_origin(),
                self.get_name(),
                self.get_version(),
                self.get_release()
            )
        } else if self.has_version() {
            write!(
                f,
                "{}/{}/{}",
                self.get_origin(),
                self.get_name(),
                self.get_version()
            )
        } else {
            write!(f, "{}/{}", self.get_origin(), self.get_name())
        }
    }
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state = match *self {
            ProcessState::Down => "down",
            ProcessState::Up => "up",
        };
        write!(f, "{}", state)
    }
}

impl fmt::Display for ServiceGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut value = format!("{}.{}", self.get_service(), self.get_group());
        if self.has_application_environment() {
            value.insert_str(0, &format!("{}#", self.get_application_environment()));
        }
        if self.has_organization() {
            value.push_str(&format!("@{}", self.get_organization()));
        }
        write!(f, "{}", value)
    }
}

// JW TODO: These trait implementations to provide translations between protocol messages and
// concrete Rust types defined in the core crate will go away eventually. We need to put the
// core crate back into the Supervisor's repository and untangle our dependency hell before
// that can happen.

impl From<hcore::service::ApplicationEnvironment> for ApplicationEnvironment {
    fn from(app_env: hcore::service::ApplicationEnvironment) -> Self {
        let mut proto = ApplicationEnvironment::new();
        proto.set_application(app_env.application().to_string());
        proto.set_environment(app_env.environment().to_string());
        proto
    }
}

impl From<package::PackageIdent> for PackageIdent {
    fn from(ident: package::PackageIdent) -> Self {
        let mut proto = PackageIdent::new();
        proto.set_origin(ident.origin);
        proto.set_name(ident.name);
        if let Some(version) = ident.version {
            proto.set_version(version);
        }
        if let Some(release) = ident.release {
            proto.set_release(release);
        }
        proto
    }
}

impl Into<package::PackageIdent> for PackageIdent {
    fn into(mut self) -> package::PackageIdent {
        let version = if self.has_version() {
            Some(self.take_version())
        } else {
            None
        };
        let release = if self.has_release() {
            Some(self.take_release())
        } else {
            None
        };
        package::PackageIdent::new(self.take_origin(), self.take_name(), version, release)
    }
}

impl From<manager::service::ProcessState> for ProcessState {
    fn from(other: manager::service::ProcessState) -> Self {
        match other {
            manager::service::ProcessState::Down => ProcessState::Down,
            manager::service::ProcessState::Up => ProcessState::Up,
        }
    }
}

impl From<manager::ProcessStatus> for ProcessStatus {
    fn from(other: manager::ProcessStatus) -> Self {
        let mut proto = ProcessStatus::new();
        proto.set_elapsed(other.elapsed.num_seconds());
        proto.set_state(other.state.into());
        if let Some(pid) = other.pid {
            proto.set_pid(pid);
        }
        proto
    }
}

impl From<manager::service::ServiceBind> for ServiceBind {
    fn from(bind: manager::service::ServiceBind) -> Self {
        let mut proto = ServiceBind::new();
        proto.set_name(bind.name);
        proto.set_service_group(bind.service_group.into());
        proto
    }
}

impl From<hcore::service::ServiceGroup> for ServiceGroup {
    fn from(service_group: hcore::service::ServiceGroup) -> Self {
        let mut proto = ServiceGroup::new();
        if let Some(app_env) = service_group.application_environment() {
            proto.set_application_environment(app_env.into());
        }
        proto.set_group(service_group.group().to_string());
        proto.set_service(service_group.service().to_string());
        if let Some(organization) = service_group.org() {
            proto.set_organization(organization.to_string());
        }
        proto
    }
}

impl From<manager::ServiceStatus> for ServiceStatus {
    fn from(other: manager::ServiceStatus) -> Self {
        let mut proto = ServiceStatus::new();
        proto.set_ident(other.pkg.ident.into());
        proto.set_process(other.process.into());
        proto.set_service_group(other.service_group.into());
        if let Some(composite) = other.composite {
            proto.set_composite(composite);
        }
        proto
    }
}

impl Into<manager::service::ServiceBind> for ServiceBind {
    fn into(mut self) -> manager::service::ServiceBind {
        manager::service::ServiceBind {
            name: self.take_name(),
            service_group: self.take_service_group().into(),
        }
    }
}

impl Into<hcore::service::ServiceGroup> for ServiceGroup {
    fn into(mut self) -> hcore::service::ServiceGroup {
        let app_env = if self.has_application_environment() {
            Some(
                hcore::service::ApplicationEnvironment::new(
                    self.get_application_environment().get_application(),
                    self.get_application_environment().get_environment(),
                ).unwrap(),
            )
        } else {
            None
        };
        let service = self.take_service();
        let group = self.take_group();
        let organization = if self.has_organization() {
            Some(self.get_organization())
        } else {
            None
        };
        hcore::service::ServiceGroup::new(app_env.as_ref(), service, group, organization).unwrap()
    }
}

impl Identifiable for PackageIdent {
    fn origin(&self) -> &str {
        self.get_origin()
    }

    fn name(&self) -> &str {
        self.get_name()
    }

    fn version(&self) -> Option<&str> {
        if self.has_version() {
            Some(self.get_version())
        } else {
            None
        }
    }

    fn release(&self) -> Option<&str> {
        if self.has_release() {
            Some(self.get_release())
        } else {
            None
        }
    }
}
