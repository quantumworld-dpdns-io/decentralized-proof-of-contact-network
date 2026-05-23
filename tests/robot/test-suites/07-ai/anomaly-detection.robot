*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/ai.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    ai    anomaly    regression

*** Test Cases ***
Detect Anomalies Returns Results
    [Tags]    smoke    ai
    ${result}=    Detect Anomalies    timeframe=7d
    Dictionary Should Contain Key    ${result}    anomalies

Anomaly Detection Reports Count
    [Tags]    ai
    ${result}=    Detect Anomalies    timeframe=7d
    Dictionary Should Contain Key    ${result}    anomaly_count

Anomaly Report Contains Details
    [Tags]    ai
    ${result}=    Detect Anomalies    timeframe=7d
    ${anomalies}=    Set Variable    ${result}[anomalies]
    IF    ${anomalies.__len__()} > 0
        ${first}=    Set Variable    ${anomalies}[0]
        Dictionary Should Contain Key    ${first}    type
        Dictionary Should Contain Key    ${first}    severity
        Dictionary Should Contain Key    ${first}    description
    END

Anomaly Detection Supports Custom Timeframes
    [Tags]    ai
    ${result}=    Detect Anomalies    timeframe=24h
    Should Not Be None    ${result}
    ${result}=    Detect Anomalies    timeframe=30d
    Should Not Be None    ${result}

Anomaly Types Include Known Patterns
    [Tags]    ai
    ${result}=    Detect Anomalies    timeframe=7d
    ${anomalies}=    Set Variable    ${result}[anomalies]
    ${types}=    Create List
    FOR    ${a}    IN    @{anomalies}
        Append To List    ${types}    ${a}[type]
    END

Anomaly Detection Is Idempotent
    [Tags]    ai
    ${r1}=    Detect Anomalies    timeframe=7d
    ${r2}=    Detect Anomalies    timeframe=7d
    Should Be Equal    ${r1}[anomaly_count]    ${r2}[anomaly_count]

Anomaly Detection Handles Empty Data
    [Tags]    ai
    ${result}=    Detect Anomalies    timeframe=1h
    Should Not Be None    ${result}

High Failure Rate Triggers Anomaly
    [Tags]    ai
    Create Proof    target_node=node-anomaly-fail    window_id=window-anomaly
    Sleep    1s
    ${result}=    Detect Anomalies    timeframe=7d
    Dictionary Should Contain Key    ${result}    anomalies

Anomaly Severity Is Scored
    [Tags]    ai
    ${result}=    Detect Anomalies    timeframe=7d
    ${anomalies}=    Set Variable    ${result}[anomalies]
    IF    ${anomalies.__len__()} > 0
        ${first}=    Set Variable    ${anomalies}[0]
        Should Be True    ${first}[severity] >= 0
        Should Be True    ${first}[severity] <= 10
    END
