// Copyright 2024 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use jj_cli::cli_util::CliRunner;
use jj_cli::commit_templater::{CommitTemplateLanguageExtension, CommitTemplatePropertyKind};
use jj_cli::template_builder::{BuildContext, CoreTemplatePropertyKind};
use jj_cli::template_parser::{
    ExpressionKind, ExpressionNode, FunctionCallNode, TemplateParseError, TemplateParseResult,
};
use jj_cli::templater::{TemplateFunction, TemplateProperty};
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;

struct HexCounter;

fn num_digits_in_id(commit: Commit) -> i64 {
    let mut count = 0;
    for ch in commit.id().hex().chars() {
        if ch.is_ascii_digit() {
            count += 1;
        }
    }
    count
}

fn num_char_in_id(commit: Commit, ch_match: char) -> i64 {
    let mut count = 0;
    for ch in commit.id().hex().chars() {
        if ch == ch_match {
            count += 1;
        }
    }
    count
}

impl CommitTemplateLanguageExtension for HexCounter {
    fn build_commit_property_opt<'repo>(
        &self,
        property: Box<dyn TemplateProperty<Commit, Output = Commit> + 'repo>,
        name: &str,
    ) -> Result<
        CommitTemplatePropertyKind<'repo>,
        Box<dyn TemplateProperty<Commit, Output = Commit> + 'repo>,
    > {
        match name {
            "num_digits_in_id" => Ok(CommitTemplatePropertyKind::Core(
                CoreTemplatePropertyKind::Integer(Box::new(TemplateFunction::new(
                    property,
                    num_digits_in_id,
                ))),
            )),
            _ => Err(property),
        }
    }

    fn build_commit_function<'repo>(
        &self,
        _build_ctx: &BuildContext<CommitTemplatePropertyKind<'repo>>,
        self_property: Box<dyn TemplateProperty<Commit, Output = Commit> + 'repo>,
        function: &FunctionCallNode,
    ) -> TemplateParseResult<CommitTemplatePropertyKind<'repo>> {
        match function.name {
            "num_char_in_id" => match &function.args[..] {
                [ExpressionNode {
                    kind: ExpressionKind::String(string),
                    span: _,
                }] => {
                    let chars: Vec<_> = string.chars().collect();
                    if chars.len() != 1 {
                        return Err(TemplateParseError::invalid_arguments(
                            function,
                            "Expected single character argument",
                        ));
                    }
                    let char = chars[0];
                    Ok(CommitTemplatePropertyKind::Core(
                        CoreTemplatePropertyKind::Integer(Box::new(TemplateFunction::new(
                            self_property,
                            move |commit| num_char_in_id(commit, char),
                        ))),
                    ))
                }
                _ => Err(TemplateParseError::invalid_arguments(
                    function,
                    "Expected singular string argument",
                )),
            },
            _ => Err(TemplateParseError::no_such_function(function)),
        }
    }
}

fn main() -> std::process::ExitCode {
    CliRunner::init()
        .set_commit_templater_extension(Box::new(HexCounter))
        .run()
}
