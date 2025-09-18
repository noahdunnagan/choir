# Choir - Multi-Agent AI Analysis System

> Preface

This project is months old and just my "lets do something to learn a random thing!" project. I 100% didn't employ best pracitces in certain places but I'm alright with that.

In timeline of "how I want projects to look" this is in phase 1. "Okay it works and does what I want. Next is to make it look pretty then finally optimize."



## Now, what is Choir?


Choir is a Rust-powered API that uses multiple AI agents to analyze complex queries. It can fetch webpage content and coordinate 5 specialized AI agents to provide comprehensive analysis from different perspectives.

## How It Works

1. **Query Analysis**: Detects URLs in your query and fetches their content using Firecrawl
2. **Task Master**: Creates 5 distinct analytical approaches for your query
3. **Agent Coordination**: Deploys 5 specialized AI agents (Direct Analyst, Critical Evaluator, Context Specialist, Creative Interpreter, Synthesis Expert)
4. **Assessment**: A task master (chorus) evaluates all agent responses and provides the best synthesis
5. **Final Summary**: Returns a clear, comprehensive answer to your original question

## Quick Start

### Prerequisites
- Rust (latest stable)
- OpenAI API key
- Firecrawl API key

### Environment Setup
Create a `.env` file:
```
PORT=8081
API_KEY=your-api-key-here (bearer auth for requests)
OAI_KEY=your-openai-api-key
FC_KEY=your-firecrawl-api-key
```

### Run Locally
```bash
cargo run
```

### Test It Out
```bash
curl -X POST http://localhost:8081/choir \
  -H "Authorization: Bearer your-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is the George Lucas quote used here? https://noahdunnagan.com/thoughts/design"
  }'
```

## API Endpoints

### Choir Analysis
- **Method**: POST
- **Path**: `/choir`
- **Auth**: Bearer token required

**Request Body**:
```json
{
  "query": "Your question or analysis request",
  "json_schema": null
}
```

**Example Queries**:
- Analyze websites: `"What are the main points in https://example.com/article?"`
- Complex questions: `"Compare the pros and cons of different approaches to..."`
- Research tasks: `"What can you tell me about..."`
