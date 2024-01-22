use crate::frontend::ast::*;
use crate::session::Session;

pub struct CodeGenerator<'sess> {
    session: &'sess Session,
    output: String,
}

impl<'sess> CodeGenerator<'sess> {
    pub fn new(session: &'sess Session) -> Self {
        Self {
            session,
            output: String::new(),
        }
    }

    pub fn run(mut self, module: &Module) -> String {
        match &module.item {
            Item::FuncDecl(func) => self.gen_func_decl(func),
            Item::ParseError => unreachable!(),
        };

        self.output
    }

    fn gen_func_decl(&mut self, f: &FuncDecl) {
        let name = self.session.lookup_str(f.name);

        self.push_line(0, format!(".globl {name}"));
        self.push_line(0, format!("{name}:"));

        self.gen_stmt(&f.statement);
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(expr) => {
                self.gen_expr(expr);

                self.push_line(1, "ret");
            }
            Stmt::ParseError => unreachable!(),
        }
    }

    /// Outputs to `rax`.
    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Constant(n) => self.push_line(1, format!("movq ${n}, %rax")),
        }
    }

    fn push_line(&mut self, indent: u8, s: impl AsRef<str>) {
        const INDENT: &str = "    ";

        for _ in 0..indent {
            self.output.push_str(INDENT);
        }

        self.output.push_str(s.as_ref());
        self.output.push('\n');
    }
}
