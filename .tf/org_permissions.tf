import {
  id = "chaotic-ground"
  to = github_actions_organization_workflow_permissions.this
}
resource "github_actions_organization_workflow_permissions" "this" {
  organization_slug = "chaotic-ground"
  # 조직 차원 기본값이 read 로 잠겨 있으면 리포별로 write 를 요청해도 409(Write
  # permissions for workflows are disabled by the organization)로 거부된다.
  # release-please 같은 워크플로가 PR 을 열려면 조직 기본값 자체가 write 여야 한다.
  default_workflow_permissions     = "write"
  can_approve_pull_request_reviews = true
}
