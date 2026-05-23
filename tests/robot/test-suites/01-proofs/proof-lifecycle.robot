*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    proof    lifecycle    regression

*** Test Cases ***
Proof Lifecycle Create To Verify
    [Tags]    smoke    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-1    window_id=window-lifecycle
    ${id}=    Set Variable    ${proof}[id]
    ${status}=    Proof Status Should Be    ${id}    pending
    ${result}=    Verify Proof    ${id}
    Should Be True    ${result}[all_passed]

Sign Proof Transitions Status
    [Tags]    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-2    window_id=window-lifecycle
    ${result}=    Sign Proof    ${proof}[id]
    Dictionary Should Contain Key    ${result}    signature

Sign Proof Updates Signature Field
    [Tags]    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-3    window_id=window-lifecycle
    ${before}=    Set Variable    ${proof}[signature]
    ${result}=    Sign Proof    ${proof}[id]
    Should Not Be Equal    ${result}[signature]    ${before}

Delete Proof
    [Tags]    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-4    window_id=window-lifecycle
    Delete Proof    ${proof}[id]
    Run Keyword And Expect Error    *404*    Get Proof    ${proof}[id]

Delete Non Existent Proof
    [Tags]    lifecycle    negative
    ${fake_id}=    Generate UUID
    Run Keyword And Expect Error    *404*    Delete Proof    ${fake_id}

Update Proof Metadata
    [Tags]    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-5    window_id=window-lifecycle
    ${updated}=    Update Proof Metadata    ${proof}[id]    proof_purpose=updated-purpose    confidence_score=0.8
    Should Be Equal    ${updated}[metadata][proof_purpose]    updated-purpose

Proof Has Creation Timestamp
    [Tags]    lifecycle
    ${before}=    Get Current Date
    ${proof}=    Create Proof    target_node=node-lifecycle-6    window_id=window-lifecycle
    ${after}=    Get Current Date
    Should Be True    ${before} <= ${proof}[timestamp]    Timestamp should be after test start
    Should Be True    ${proof}[timestamp] <= ${after}    Timestamp should be before test end

Proof Can Be Retrieved After Creation
    [Tags]    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-7    window_id=window-lifecycle
    ${id}=    Set Variable    ${proof}[id]
    ${retrieved}=    Get Proof    ${id}
    Should Be Equal    ${retrieved}[id]    ${id}

Proof Status Transitions Are Valid
    [Tags]    lifecycle
    ${proof}=    Create Proof    target_node=node-lifecycle-8    window_id=window-lifecycle
    ${statuses}=    List Proof Statuses
    ${initial}=    Get Proof    ${proof}[id]
    Should Contain    ${statuses}    ${initial}[status]

Proof Cannot Be Deleted Twice
    [Tags]    lifecycle    negative
    ${proof}=    Create Proof    target_node=node-lifecycle-9    window_id=window-lifecycle
    Delete Proof    ${proof}[id]
    Run Keyword And Expect Error    *404*    Delete Proof    ${proof}[id]
