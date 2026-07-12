import {
  id = "geulbus"
  to = github_workflow_repository_permissions.this
}
resource "github_workflow_repository_permissions" "this" {
  repository = github_repository.this.name
  # GITHUB_TOKEN 기본 권한. 저장소 기본값이 read 면 각 워크플로 파일의
  # permissions: 블록이 write 를 요청해도 read 로 깎인다.
  default_workflow_permissions = "write"
  # release-please 가 릴리스 PR 을 열려면 필요(정확히 이 스위치가 없으면
  # "GitHub Actions is not permitted to create or approve pull requests").
  can_approve_pull_request_reviews = true
}
