*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/ai.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    ai    analysis    regression

*** Test Cases ***
Analyze Proof Returns Summary
    [Tags]    smoke    ai
    ${proof}=    Create Proof    target_node=node-analysis-1    window_id=window-analysis
    ${analysis}=    Analyze Proof Should Succeed    ${proof}[id]

Analysis Contains Proof Details
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-2    window_id=window-analysis
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    proof_id
    Should Be Equal    ${analysis}[proof_id]    ${proof}[id]

Analysis Contains Proving Node Info
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-3    window_id=window-analysis
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    proving_node

Analysis Contains Target Node Info
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-4    window_id=window-analysis
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    target_node

Analysis Contains Confidence Score
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-5    window_id=window-analysis    confidence_score=0.85
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    confidence_score

Analysis Returns Timestamp
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-6    window_id=window-analysis
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    analyzed_at

Analyze Non Existent Proof
    [Tags]    ai    negative
    ${fake_id}=    Generate UUID
    Run Keyword And Expect Error    *404*    Analyze Proof Pattern    ${fake_id}

Analysis Returns Verification Status
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-8    window_id=window-analysis
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    verification_status

Analysis Returns Pattern Insights
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-9    window_id=window-analysis
    ${analysis}=    Analyze Proof Pattern    ${proof}[id]
    Dictionary Should Contain Key    ${analysis}    insights

Analysis Is Deterministic
    [Tags]    ai
    ${proof}=    Create Proof    target_node=node-analysis-10    window_id=window-analysis
    ${a1}=    Analyze Proof Pattern    ${proof}[id]
    ${a2}=    Analyze Proof Pattern    ${proof}[id]
    Should Be Equal    ${a1}[summary]    ${a2}[summary]
