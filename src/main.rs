use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod binary;
mod vm;
use crate::vm::instruction::Instruction;
use crate::vm::opcodes::*;

mod api;
mod state;
use crate::api::{consts::*, LuaAPI};
use crate::state::LuaState;

fn main() {
    let mut ls = LuaState::new();
    ls.push_boolean(true); print_stack(&ls);
    ls.push_integer(10); print_stack(&ls);
    ls.push_nil(); print_stack(&ls);
    ls.push_string("hello".to_string()); print_stack(&ls);
    ls.push_value(-4); print_stack(&ls);
    ls.replace(3); print_stack(&ls);
    ls.set_top(6); print_stack(&ls);
    ls.remove(-3); print_stack(&ls);
    ls.set_top(-5); print_stack(&ls);
}

fn print_stack(ls: &LuaState) {
    let top = ls.get_top();
    for i in 1..=top {
        match ls.type_id(i) {
            LUA_TBOOLEAN => print!("[{}]", ls.to_boolean(i)),
            LUA_TNUMBER => print!("[{}]", ls.to_number(i)),
            LUA_TSTRING => print!("[\"{}\"]", ls.to_string(i)),
            t => print!("[{}]", ls.type_name(t)),
        }
    }
    println!("");
}

/*
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let mut f = File::open(filename)?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        let proto = binary::undump(data);
        list(&proto);
    } else {
        println!("Please input file name.");
    }

    Ok(())
}

fn list(proto: &binary::chunk::Prototype) {
    print_header(proto);
    print_code(proto);
    print_detail(proto);
    for p in proto.protos.iter() {
        list(p);
    }
}

fn print_header(proto: &binary::chunk::Prototype) {
    let func_type = if proto.line_defined > 0 { "function" } else { "main" };
    let vararg_flag = if proto.is_vararg > 0 { "+" } else { "" };
    println!(
        "\n{} <{}:{}, {}> ({} instructions)",
        func_type,
        proto.source.as_ref().map(|x| x.as_str()).unwrap_or(""),
        proto.line_defined,
        proto.last_line_defined,
        proto.code.len()
    );
    print!(
        "{}{} params, {} slots, {} upvalues, ",
        proto.num_params,
        vararg_flag,
        proto.max_stack_size,
        proto.upvalues.len()
    );
    println!(
        "{} locals, {} constants, {} functions",
        proto.loc_vars.len(),
        proto.constants.len(),
        proto.protos.len()
    );
}

fn print_code(proto: &binary::chunk::Prototype) {
    for (pc, c) in proto.code.iter().enumerate() {
        let line = if proto.line_info.len() > 0 {
            format!("{}", proto.line_info[pc as usize])
        } else {
            "-".to_string()
        };
        print!("\t{}\t[{}]\t{} \t", pc + 1, line, (*c).opname());
        print_oprands(*c);
        println!("");
    }
}

fn print_oprands(i: u32) {
    match i.opmode() {
        OP_MODE_ABC => {
            let (a, b, c) = i.abc();
            print!("{}", a);
            if i.b_mode() != OP_ARG_N {
                if b > 0xFF {
                    print!(" {}", -1 - (b & 0xFF));
                } else {
                    print!(" {}", b);
                }
            }
            if i.c_mode() != OP_ARG_N {
                if c > 0xFF {
                    print!(" {}", -1 - (c & 0xFF));
                } else {
                    print!(" {}", c);
                }
            }
        }
        OP_MODE_ABX => {
            let (a, bx) = i.a_bx();
            print!(" {}", a);
            if i.b_mode() == OP_ARG_K {
                print!(" {}", -1 - bx);
            } else if i.b_mode() == OP_ARG_U {
                print!(" {}", bx);
            }
        }
        OP_MODE_ASBX => {
            let (a, sbx) = i.a_sbx();
            print!("{} {}", a, sbx);
        }
        OP_MODE_AX => {
            let ax = i.ax();
            print!("{}", -1 - ax);
        }
        _ => unreachable!(),
    };
}

fn print_detail(proto: &binary::chunk::Prototype) {
    fn constant_to_string(k: &binary::chunk::Constant) -> String {
        match k {
            binary::chunk::Constant::Nil => "nil".to_string(),
            binary::chunk::Constant::Boolean(b) => format!("{}", b),
            binary::chunk::Constant::Integer(i) => format!("{}", i),
            binary::chunk::Constant::Number(f) => format!("{}", f),
            binary::chunk::Constant::String(s) => format!("\"{}\"", s),
            _ => "?".to_string(),
        }
    }

    fn upvalue_name(proto: &binary::chunk::Prototype, idx: usize) -> &str {
        if proto.upvalue_names.len() > 0 {
            &proto.upvalue_names[idx]
        } else {
            "-"
        }
    }

    println!("constants ({}):", proto.constants.len());
    for (i, k) in proto.constants.iter().enumerate() {
        println!("\t{}\t{}", i + 1, constant_to_string(k));
    }

    println!("locals ({}):", proto.loc_vars.len());
    for (i, loc_var) in proto.loc_vars.iter().enumerate() {
        println!(
            "\t{}\t{}\t{}\t{}",
            i,
            loc_var.var_name,
            loc_var.start_pc + 1,
            loc_var.end_pc + 1
        );
    }

    println!("upvalues ({}):", proto.upvalues.len());
    for (i, upval) in proto.upvalues.iter().enumerate() {
        println!("\t{}\t{}\t{}\t{}", i, upvalue_name(proto, i), upval.instack, upval.idx);
    }
}
*/