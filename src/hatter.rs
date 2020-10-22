//! Helper functions/structs for working with Hatter templates.

use {
    crate::utils::html_encode,
    hatter,
    std::{io, ops},
    vial::asset,
};

pub struct Hatter {
    env: hatter::Env,
}

impl ops::Deref for Hatter {
    type Target = hatter::Env;
    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

impl ops::DerefMut for Hatter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

impl Hatter {
    pub fn new() -> Hatter {
        Hatter {
            env: hatter::Env::new(),
        }
    }

    /// Render template and fallback to a nice HTML error.
    pub fn render(&mut self, path: &str) -> Result<String, io::Error> {
        use hatter::ErrorKind::*;

        let src = asset::to_string(path)?;
        match self.env.render(&src) {
            Ok(out) => Ok(out.into()),
            Err(err) => match err.kind {
                ParseError | SyntaxError | RuntimeError => {
                    let (errline, errcol) = hatter::line_and_col(&src, err.pos);
                    Ok(format!(
                        "<html><body>
                        <h2>{:?}: {}</h2>
                        <h3>{}: line {}, col {}</h3>
                        <pre>{}",
                        err.kind,
                        err.details,
                        path,
                        errline,
                        errcol,
                        html_encode(&src)
                            .split('\n')
                            .enumerate()
                            .map(|(i, line)| if i + 1 == errline {
                                let errcol = if errcol > 0 { errcol - 1 } else { 0 };
                                format!(
                                    "<b>{}</b>\n<span style='color:red'>{}{}</span>\n",
                                    line,
                                    " ".repeat(errcol),
                                    "^".repeat(err.len)
                                )
                            } else {
                                format!("{}\n", line)
                            })
                            .collect::<String>(),
                    ))
                }

                ArgNotFound | WrongArgType => Ok(format!(
                    "<html><body><h1>{}</h1><h3 style='color:red'>{:?}</h3>",
                    path, err.details
                )),
                _ => Ok(format!(
                    "<html><body><h1>{}</h1><h3 style='color:red'>{:?}</h3>",
                    path, err
                )),
            },
        }
    }
}
