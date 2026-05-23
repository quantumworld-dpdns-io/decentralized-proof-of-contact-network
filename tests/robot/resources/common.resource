*** Settings ***
Library    Collections
Library    String
Library    DateTime
Library    OperatingSystem
Library    ../libraries/CryptoHelper.py
Library    ../libraries/ProofLibrary.py
Library    ../libraries/NetworkSimulator.py
Library    ../libraries/SecurityHelper.py

*** Variables ***
${API_BASE_URL}    http://localhost:3000
${WS_URL}    ws://localhost:3000/ws
${DEFAULT_AUTH_TOKEN}    test-bearer-token-for-development
${DEFAULT_NODE_ID}    dev-node-001
${PROTOCOL_VERSION}    1.0
${POLL_INTERVAL}    0.5
${POLL_TIMEOUT}    30
${LONG_TIMEOUT}    60

*** Keywords ***
Setup Test Environment
    Log    Setting up test environment...
    Set Suite Variable    ${PROOF_LIB}    ProofLibrary
    Set Suite Variable    ${NETWORK_SIM}    NetworkSimulator
    Set Suite Variable    ${CRYPTO}    CryptoHelper
    Set Suite Variable    ${SECURITY}    SecurityHelper

Teardown Test Environment
    Log    Tearing down test environment...

Wait Until Keyword Succeeds With Timeout
    [Arguments]    ${keyword}    ${timeout}=${POLL_TIMEOUT}    ${interval}=${POLL_INTERVAL}    @{args}
    ${end}=    Evaluate    time.time() + ${timeout}    time
    WHILE    True
        ${passed}=    Run Keyword And Return Status    ${keyword}    @{args}
        IF    ${passed}    BREAK
        ${now}=    Evaluate    time.time()    time
        IF    ${now} > ${end}    FAIL    Timed out waiting for '${keyword}' to succeed
        Sleep    ${interval}
    END

Generate Random String
    [Arguments]    ${length}=8
    ${result}=    Generate Random String    ${length}    [LOWER]
    RETURN    ${result}

Generate UUID
    ${uuid}=    Evaluate    str(__import__('uuid').uuid4())
    RETURN    ${uuid}

Generate Timestamp ISO
    ${ts}=    Evaluate    __import__('datetime').datetime.now(__import__('datetime').timezone.utc).isoformat()
    RETURN    ${ts}

Validate Proof Structure
    [Arguments]    ${proof}
    Dictionary Should Contain Key    ${proof}    id
    Dictionary Should Contain Key    ${proof}    proving_node
    Dictionary Should Contain Key    ${proof}    target_node
    Dictionary Should Contain Key    ${proof}    timestamp
    Dictionary Should Contain Key    ${proof}    signature
    Dictionary Should Contain Key    ${proof}    metadata

Validate Window Structure
    [Arguments]    ${window}
    Dictionary Should Contain Key    ${window}    id
    Dictionary Should Contain Key    ${window}    start_time
    Dictionary Should Contain Key    ${window}    end_time
    Dictionary Should Contain Key    ${window}    window_type

Create Test Proof
    [Arguments]    ${target}=node-${EMPTY}    ${window}=window-${EMPTY}    ${purpose}=test
    IF    '${target}' == 'node-'    ${target}=    Set Variable    node-${uuid}
    IF    '${window}' == 'window-'    ${window}=    Set Variable    window-${uuid}
    ${proof}=    Create Proof    target_node=${target}    window_id=${window}    purpose=${purpose}
    RETURN    ${proof}

Should Be Valid Proof
    [Arguments]    ${proof}
    Validate Proof Structure    ${proof}
    Should Not Be Empty    ${proof}[id]
    Should Not Be Empty    ${proof}[proving_node]
    Should Not Be Empty    ${proof}[target_node]
    Should Not Be Empty    ${proof}[timestamp]
    Should Be Equal    ${proof}[metadata][protocol_version]    ${PROTOCOL_VERSION}

Should Be Valid Window
    [Arguments]    ${window}
    Validate Window Structure    ${window}
    Should Not Be Empty    ${window}[id]
    Should Not Be Empty    ${window}[start_time]
    Should Not Be Empty    ${window}[end_time]
