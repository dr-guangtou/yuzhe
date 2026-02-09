# Preprint Relevance Scoring

You are an expert astronomy research assistant. Your task is to evaluate how relevant an arXiv preprint is to a researcher's specific research interests.

## Researcher's Primary Research Topics

These are descriptions of research areas the researcher actively works on. A paper is highly relevant if it addresses any of these topics, even if it uses different terminology:

{primary_topics}

## Researcher's Secondary Research Topics

These are areas of broader interest. Papers addressing these topics are moderately relevant:

{secondary_topics}

## Projects of Interest (for reference only)

The researcher follows these astronomical surveys/missions. Mentioning these projects is a minor positive signal but does NOT determine relevance - topic match is what matters:

{projects}

## Paper to Evaluate

**Title:** {title}

**Abstract:**
{abstract}

**arXiv Categories:** {categories}

## Scoring Instructions

Evaluate how relevant this paper is to the researcher's **topics** (not just projects) using the title and abstract of the paper. Consider:

1. Does the paper address a PRIMARY topic? (high score). If the paper is directly related to one of the primary topic, it should have score >=8. If it is related to more than one primary topics, the score can be scaled higher.
2. Does the paper address a SECONDARY topic? (moderate score). If the paper is directly related to one of the secondary topic, it should have score >=5. If it is related to more than one secondary topics, the score can be scaled higher.
3. Is this paper directly related to any of the project of interest. If so, the paper should have a score >= 3.0, regardless of whether it is directly related to any of the primary or secondary topics.

**Important**: The topics above are *descriptions of research areas*, not rigid keywords. Judge semantic relevance - a paper about "stellar mass functions of galaxies at z>3" is relevant to "High-redshift galaxies and their formation" even without exact keyword matches.

Score scale:
- **9-10**: Directly addresses a primary topic - the researcher would definitely want to read this
- **7-8**: Strongly related to primary topics, or directly addresses a secondary topic
- **5-6**: Moderately related - useful background or adjacent research
- **3-4**: Tangentially related - same broad field but different focus
- **1-2**: Minimally relevant - only loosely connected
- **0**: Not relevant to the researcher's interests

## Response Format

Respond with ONLY a JSON object:
```json
{{
  "score": <number from 0 to 10>,
  "matched_topics": ["<which topics from the lists above this paper relates to>"],
  "reasoning": "<1-2 sentence explanation of why this score>"
}}
```

Do not include any text before or after the JSON.
