use std::env;

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOperator {
    And, // &&
    Or,  // ||
    Seq, // ;
}

#[derive(Debug, PartialEq, Clone)]
pub enum RedirectionKind {
    Write,  // >
    Append, // >>
    Read,   // <
}

#[derive(Debug, PartialEq, Clone)]
pub struct Redirection {
    pub kind: RedirectionKind,
    pub target: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandSpec {
    pub args: Vec<String>,
    pub redirections: Vec<Redirection>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pipeline {
    pub commands: Vec<CommandSpec>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Job {
    pub pipeline: Pipeline,
    pub next_op: Option<LogicalOperator>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandAst {
    pub jobs: Vec<Job>,
}

pub fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    var_name.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            if !var_name.is_empty() {
                if let Ok(val) = env::var(&var_name) {
                    result.push_str(&val);
                } else {
                    result.push('$');
                    result.push_str(&var_name);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Word(String),
    OpAnd,       // &&
    OpOr,        // ||
    OpSeq,       // ;
    Pipe,        // |
    RedirWrite,  // >
    RedirAppend, // >>
    RedirRead,   // <
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    while let Some(&ch) = chars.peek() {
        if in_quotes {
            chars.next();
            if ch == quote_char {
                in_quotes = false;
            } else {
                current.push(ch);
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                chars.next();
                in_quotes = true;
                quote_char = ch;
            }
            ' ' | '\t' | '\r' | '\n' => {
                chars.next();
                if !current.is_empty() {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                }
            }
            '&' => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                }
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::OpAnd);
                } else {
                    current.push('&');
                }
            }
            '|' => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                }
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::OpOr);
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            ';' => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                }
                chars.next();
                tokens.push(Token::OpSeq);
            }
            '>' => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                }
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::RedirAppend);
                } else {
                    tokens.push(Token::RedirWrite);
                }
            }
            '<' => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                }
                chars.next();
                tokens.push(Token::RedirRead);
            }
            _ => {
                chars.next();
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(Token::Word(current));
    }

    tokens
}

pub fn parse_ast(input: &str) -> CommandAst {
    let tokens = tokenize(input);
    let mut jobs = Vec::new();

    let mut current_pipeline_cmds = Vec::new();
    let mut current_args = Vec::new();
    let mut current_redirs = Vec::new();

    let mut iter = tokens.into_iter().peekable();

    while let Some(tok) = iter.next() {
        match tok {
            Token::Word(w) => {
                current_args.push(w);
            }
            Token::RedirWrite => {
                if let Some(Token::Word(target)) = iter.next() {
                    current_redirs.push(Redirection {
                        kind: RedirectionKind::Write,
                        target,
                    });
                }
            }
            Token::RedirAppend => {
                if let Some(Token::Word(target)) = iter.next() {
                    current_redirs.push(Redirection {
                        kind: RedirectionKind::Append,
                        target,
                    });
                }
            }
            Token::RedirRead => {
                if let Some(Token::Word(target)) = iter.next() {
                    current_redirs.push(Redirection {
                        kind: RedirectionKind::Read,
                        target,
                    });
                }
            }
            Token::Pipe => {
                if !current_args.is_empty() || !current_redirs.is_empty() {
                    current_pipeline_cmds.push(CommandSpec {
                        args: std::mem::take(&mut current_args),
                        redirections: std::mem::take(&mut current_redirs),
                    });
                }
            }
            Token::OpAnd | Token::OpOr | Token::OpSeq => {
                if !current_args.is_empty() || !current_redirs.is_empty() {
                    current_pipeline_cmds.push(CommandSpec {
                        args: std::mem::take(&mut current_args),
                        redirections: std::mem::take(&mut current_redirs),
                    });
                }

                if !current_pipeline_cmds.is_empty() {
                    let op = match tok {
                        Token::OpAnd => LogicalOperator::And,
                        Token::OpOr => LogicalOperator::Or,
                        Token::OpSeq => LogicalOperator::Seq,
                        _ => unreachable!(),
                    };
                    jobs.push(Job {
                        pipeline: Pipeline {
                            commands: std::mem::take(&mut current_pipeline_cmds),
                        },
                        next_op: Some(op),
                    });
                }
            }
        }
    }

    if !current_args.is_empty() || !current_redirs.is_empty() {
        current_pipeline_cmds.push(CommandSpec {
            args: current_args,
            redirections: current_redirs,
        });
    }

    if !current_pipeline_cmds.is_empty() {
        jobs.push(Job {
            pipeline: Pipeline {
                commands: current_pipeline_cmds,
            },
            next_op: None,
        });
    }

    CommandAst { jobs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let ast = parse_ast("echo hello world");
        assert_eq!(ast.jobs.len(), 1);
        assert_eq!(ast.jobs[0].pipeline.commands.len(), 1);
        assert_eq!(
            ast.jobs[0].pipeline.commands[0].args,
            vec!["echo", "hello", "world"]
        );
        assert_eq!(ast.jobs[0].next_op, None);
    }

    #[test]
    fn test_parse_pipeline() {
        let ast = parse_ast("cat file.txt | search test");
        assert_eq!(ast.jobs.len(), 1);
        assert_eq!(ast.jobs[0].pipeline.commands.len(), 2);
        assert_eq!(
            ast.jobs[0].pipeline.commands[0].args,
            vec!["cat", "file.txt"]
        );
        assert_eq!(
            ast.jobs[0].pipeline.commands[1].args,
            vec!["search", "test"]
        );
    }

    #[test]
    fn test_parse_logical_operators() {
        let ast = parse_ast("cargo check && cargo run || echo failed");
        assert_eq!(ast.jobs.len(), 3);
        assert_eq!(ast.jobs[0].next_op, Some(LogicalOperator::And));
        assert_eq!(ast.jobs[1].next_op, Some(LogicalOperator::Or));
        assert_eq!(ast.jobs[2].next_op, None);
    }

    #[test]
    fn test_parse_redirections() {
        let ast = parse_ast("echo \"hello world\" > output.txt");
        assert_eq!(ast.jobs.len(), 1);
        assert_eq!(
            ast.jobs[0].pipeline.commands[0].args,
            vec!["echo", "hello world"]
        );
        assert_eq!(
            ast.jobs[0].pipeline.commands[0].redirections,
            vec![Redirection {
                kind: RedirectionKind::Write,
                target: "output.txt".to_string(),
            }]
        );
    }
}
