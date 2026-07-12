import {
  id = "geulbus"
  to = github_repository.this
}
resource "github_repository" "this" {
  name        = "geulbus"
  description = "날개셋(nalgaeset) 입력 설정과 호환되는 순수 Rust ibus 한글 입력기 (WIP)"
  visibility  = "public"

  has_issues      = true
  has_projects    = true
  has_wiki        = true
  has_discussions = false
  has_downloads   = true
  is_template     = false

  allow_auto_merge            = false
  allow_merge_commit          = true
  allow_rebase_merge          = true
  allow_squash_merge          = true
  allow_update_branch         = false
  merge_commit_title          = "MERGE_MESSAGE"
  merge_commit_message        = "PR_TITLE"
  squash_merge_commit_title   = "COMMIT_OR_PR_TITLE"
  squash_merge_commit_message = "COMMIT_MESSAGES"
  delete_branch_on_merge      = false

  auto_init                   = false
  archived                    = false
  archive_on_destroy          = true
  web_commit_signoff_required = false
  topics                      = []

  security_and_analysis {
    secret_scanning {
      status = "disabled"
    }
    secret_scanning_push_protection {
      status = "disabled"
    }
  }

  lifecycle {
    ignore_changes = [
      # Cannot be imported
      archive_on_destroy,
      # Deprecated
      ignore_vulnerability_alerts_during_read,
    ]
  }
}
