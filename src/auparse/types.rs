use auparse_sys::*;

// todo;; this can be autogen from lib/msg_typetab.h

#[derive(Debug, PartialEq)]
pub enum Type {
    Unknown(u32),
    Syscall,
    Cwd,
    Path,
    Proctitle,
    SystemBoot,
}

impl From<u32> for Type {
    fn from(v: u32) -> Self {
        use Type::*;

        match v {
            AUDIT_SYSCALL => Syscall,
            AUDIT_CWD => Cwd,
            AUDIT_PATH => Path,
            AUDIT_PROCTITLE => Proctitle,
            AUDIT_SYSTEM_BOOT => SystemBoot,
            _ => Unknown(v),
        }
    }
}

impl From<Type> for u32 {
    fn from(t: Type) -> Self {
        use Type::*;
        print!("+");

        match t {
            Unknown(v) => v,
            Syscall => AUDIT_SYSCALL,
            Cwd => AUDIT_CWD,
            Path => AUDIT_PATH,
            Proctitle => AUDIT_PROCTITLE,
            SystemBoot => AUDIT_SYSTEM_BOOT,
        }
    }
}
