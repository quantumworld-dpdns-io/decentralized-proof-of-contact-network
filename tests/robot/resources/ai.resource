*** Settings ***
Resource    common.resource
Resource    api.resource

*** Variables ***
${AI_QUERY_ENDPOINT}    ${API_BASE}/ai/query
${AI_ANALYZE_ENDPOINT}    ${API_BASE}/ai/analyze
${AI_ANOMALY_ENDPOINT}    ${API_BASE}/ai/anomalies
${AI_EMBED_ENDPOINT}    ${API_BASE}/ai/embed

*** Keywords ***
Query Proofs With Natural Language
    [Arguments]    ${query}    ${limit}=10
    &{payload}=    Create Dictionary    query=${query}    limit=${limit}
    ${resp}=    API Post    ${AI_QUERY_ENDPOINT}    payload=${payload}
    RETURN    ${resp.json()}

Analyze Proof Pattern
    [Arguments]    ${proof_id}
    ${resp}=    API Post    ${AI_ANALYZE_ENDPOINT}    payload={"proof_id": "${proof_id}"}
    RETURN    ${resp.json()}

Detect Anomalies
    [Arguments]    ${timeframe}=7d
    &{payload}=    Create Dictionary    timeframe=${timeframe}
    ${resp}=    API Post    ${AI_ANOMALY_ENDPOINT}    payload=${payload}
    RETURN    ${resp.json()}

Get Embedding
    [Arguments]    ${text}
    &{payload}=    Create Dictionary    text=${text}
    ${resp}=    API Post    ${AI_EMBED_ENDPOINT}    payload=${payload}
    RETURN    ${resp.json()}

NL Query Should Return Results
    [Arguments]    ${query}    ${minimum_results}=1
    ${results}=    Query Proofs With Natural Language    ${query}
    ${count}=    Get Length    ${results}
    Should Be True    ${count} >= ${minimum_results}
    RETURN    ${results}

Analyze Proof Should Succeed
    [Arguments]    ${proof_id}
    ${analysis}=    Analyze Proof Pattern    ${proof_id}
    Dictionary Should Contain Key    ${analysis}    summary
    RETURN    ${analysis}

Request Should Return Anomalies
    [Arguments]    ${timeframe}=7d
    ${anomalies}=    Detect Anomalies    ${timeframe}
    Dictionary Should Contain Key    ${anomalies}    anomalies
    RETURN    ${anomalies}
