use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod binary;

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
    let func_type = if proto.line_defined > 0 {
        "function"
    } else {
        "main"
    };
    let vararg_flag = if proto.is_vararg > 0 { "+" } else { "" };
    println!(
        "\n{} <{}:{}, {}> ({} instructions)",
        func_type,
        proto.source,
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
        println!("\t{}\t[{}]\t{:#010x}", pc + 1, line, c);
    }
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
        println!(
            "\t{}\t{}\t{}\t{}",
            i,
            upvalue_name(proto, i),
            upval.instack,
            upval.idx
        );
    }
}
