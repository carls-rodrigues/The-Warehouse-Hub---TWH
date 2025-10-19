

### How Agents Share Context

```json
{
  "project_state": {
    "project_id": "PROJ-2024-001",
    "project_name": "Task Management System",
    "client": "Acme Corp",
    "phase": "development",
    "sprint": 3,
    
    "requirements": {
      "total": 45,
      "completed": 32,
      "in_progress": 8,
      "blocked": 0,
      "requirements_list": [
        {
          "id": "REQ-001",
          "title": "User Authentication",
          "status": "completed",
          "owner": "ba-agent"
        }
      ]
    },
    
    "architecture": {
      "status": "approved",
      "tech_stack": ["React", "Node.js", "PostgreSQL"],
      "owner": "architect-agent",
      "last_updated": "ISO 8601"
    },
    
    "tasks": {
      "total": 128,
      "completed": 95,
      "in_progress": 12,
      "pending": 21,
      "tasks_list": [
        {
          "id": "TASK-001",
          "requirement_id": "REQ-001",
          "status": "completed",
          "assignee": "dev-agent-1",
          "code_review": "approved",
          "tests": "passing"
        }
      ]
    },
    
    "quality_metrics": {
      "code_coverage": 87.5,
      "security_score": "A",
      "performance_score": 92,
      "documentation_completeness": 85
    },
    
    "deployments": {
      "staging": {
        "version": "v1.2.0-beta.3",
        "deployed_at": "ISO 8601",
        "health": "healthy"
      },
      "production": {
        "version": "v1.1.0",
        "deployed_at": "ISO 8601",
        "health": "healthy"
      }
    }
  }
}
```

### Inter-Agent Communication Protocol

```
When Agent A completes work and passes to Agent B:

{
  "from_agent": "ba-agent",
  "to_agent": "architect-agent",
  "message_type": "task_completion",
  "timestamp": "ISO 8601",
  
  "completed_work": {
    "deliverable": "Requirements Document v1.0",
    "summary": "Extracted and validated 45 requirements from client brief",
    "artifacts": [
      {
        "type": "document",
        "location": "project_state.requirements",
        "version": "1.0"
      }
    ]
  },
  
  "context_for_next_agent": {
    "priority_requirements": ["REQ-001", "REQ-002", "REQ-005"],
    "constraints": ["Budget: $50k", "Timeline: 8 weeks", "Team: 2 developers"],
    "assumptions_needing_validation": [
      "Client has AWS infrastructure available",
      "Users primarily on desktop browsers"
    ],
    "questions_for_next_phase": [
      "Should we use serverless or traditional hosting?",
      "Real-time features needed - consider WebSocket architecture?"
    ]
  },
  
  "blocking_issues": [],
  
  "recommendations": [
    "Consider microservices for user auth to enable future SSO integration",
    "Plan for mobile API from start even if mobile app is future phase"
  ]
}
```

### State Persistence Strategy

```
MEMORY LAYERS:

1. PERMANENT STATE (Database/File System)
- Requirements documents
- Architecture decisions
- Source code
- Test results
- Deployment history

2. SESSION STATE (Active Project)
- Current tasks in progress
- Active code reviews
- Real-time metrics
- Agent availability

3. EPHEMERAL STATE (Individual Agent)
- Current task context
- Intermediate calculations
- Draft outputs before submission

STATE UPDATE PROTOCOL:
- Agents READ from permanent/session state
- Agents WRITE through Orchestrator (prevents conflicts)
- Orchestrator validates and commits state changes
- All agents notified of relevant state changes
```