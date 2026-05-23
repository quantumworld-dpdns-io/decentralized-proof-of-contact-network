*** Settings ***
Resource    common.resource
Resource    api.resource

*** Keywords ***
Create Proof
    [Arguments]    ${target_node}    ${window_id}    ${purpose}=test    ${confidence_score}=1.0
    ${proof}=    Create Proof Via API    ${target_node}    ${window_id}    ${purpose}    ${confidence_score}
    RETURN    ${proof}

Create Proof And Validate
    [Arguments]    ${target_node}    ${window_id}    ${purpose}=test
    ${proof}=    Create Proof    ${target_node}    ${window_id}    ${purpose}
    Should Be Valid Proof    ${proof}
    Should Be Equal    ${proof}[target_node]    ${target_node}
    RETURN    ${proof}

Get Proof
    [Arguments]    ${proof_id}
    ${proof}=    Get Proof Via API    ${proof_id}
    RETURN    ${proof}

Verify Proof
    [Arguments]    ${proof_id}    ${public_key}=${NONE}
    ${result}=    Verify Proof Via API    ${proof_id}    ${public_key}
    RETURN    ${result}

Verify Proof Should Succeed
    [Arguments]    ${proof_id}    ${public_key}=${NONE}
    ${result}=    Verify Proof    ${proof_id}    ${public_key}
    Dictionary Should Contain Key    ${result}    verified
    Should Be True    ${result}[verified]    Proof ${proof_id} should be verified
    RETURN    ${result}

Verify Proof Should Fail
    [Arguments]    ${proof_id}    ${public_key}=${NONE}
    ${result}=    Verify Proof    ${proof_id}    ${public_key}
    Dictionary Should Contain Key    ${result}    verified
    Should Not Be True    ${result}[verified]    Proof ${proof_id} should not be verified
    RETURN    ${result}

Search Proofs
    [Arguments]    ${query}=${NONE}    ${proving_node}=${NONE}    ${target_node}=${NONE}    ${window_id}=${NONE}
    ${results}=    Search Proofs Via API    query=${query}    proving_node=${proving_node}
    ...    target_node=${target_node}    window_id=${window_id}
    RETURN    ${results}

Search And Verify Results
    [Arguments]    ${query}    ${expected_count}=${NONE}    ${proving_node}=${NONE}
    ${results}=    Search Proofs    query=${query}    proving_node=${proving_node}
    IF    ${expected_count}    Should Be Equal As Integers    ${results.__len__()}    ${expected_count}
    RETURN    ${results}

Delete Proof
    [Arguments]    ${proof_id}
    Delete Proof Via API    ${proof_id}

Sign Proof
    [Arguments]    ${proof_id}    ${secret_key}=${NONE}
    ${result}=    Sign Proof Via API    ${proof_id}    ${secret_key}
    RETURN    ${result}

Update Proof Metadata
    [Arguments]    ${proof_id}    &{metadata}
    ${result}=    Update Proof Via API    ${proof_id}    ${metadata}
    RETURN    ${result}

Create Proof Chain
    [Arguments]    ${count}=3
    ${chain}=    Create Proof Chain Via API    ${count}
    RETURN    ${chain}

Get Proof Chain
    [Arguments]    ${proof_id}
    ${chain}=    Get Proof Chain Via API    ${proof_id}
    RETURN    ${chain}

Verify Chain Integrity
    [Arguments]    ${proof_id}
    ${result}=    Verify Chain Integrity Via API    ${proof_id}
    RETURN    ${result}

Create Test Proofs In Window
    [Arguments]    ${window_id}    ${count}=5    ${prefix}=node
    ${proofs}=    Create List
    FOR    ${i}    IN RANGE    ${count}
        ${target}=    Set Variable    ${prefix}-${i}
        ${proof}=    Create Proof    ${target}    ${window_id}
        Append To List    ${proofs}    ${proof}
    END
    RETURN    ${proofs}

Proof Status Should Be
    [Arguments]    ${proof_id}    ${expected_status}
    ${proof}=    Get Proof    ${proof_id}
    Should Be Equal    ${proof}[status]    ${expected_status}
