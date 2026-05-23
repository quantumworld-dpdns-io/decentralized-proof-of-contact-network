*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    smoke    health

*** Test Cases ***
Health Endpoint Returns Ok
    ${body}=    Health Check
    Should Be Equal    ${body}[status]    ok
    Should Contain    ${body}    version
    Should Contain    ${body}    uptime

Health Check Returns Timestamp
    ${body}=    Health Check
    Dictionary Should Contain Key    ${body}    timestamp
    ${ts}=    Evaluate    __import__('datetime').datetime.fromisoformat('${body}[timestamp]')
    Should Be True    ${ts} is not None    Timestamp should be valid ISO format

Health Check Returns Version String
    ${body}=    Health Check
    Should Match Regex    ${body}[version]    \\d+\\.\\d+\\.\\d+    Version should be semver

Health Check With Authentication
    ${headers}=    Get Auth Headers
    ${resp}=    GET    ${API_BASE_URL}/health    headers=${headers}
    Status Should Be    200    ${resp}

Health Check Without Authentication
    ${resp}=    GET    ${API_BASE_URL}/health
    Status Should Be    200    ${resp}    Health endpoint should be public

Health Check Returns Component Status
    ${body}=    Health Check
    Dictionary Should Contain Key    ${body}    components
    ${components}=    Set Variable    ${body}[components]
    Dictionary Should Contain Key    ${components}    api
    Dictionary Should Contain Key    ${components}    storage
    Dictionary Should Contain Key    ${components}    network

Health Check Component Status Is Up
    ${body}=    Health Check
    ${components}=    Set Variable    ${body}[components]
    Should Be Equal    ${components}[api][status]    up
    Should Be Equal    ${components}[storage][status]    up

Health Check Returns Database Status
    ${body}=    Health Check
    Dictionary Should Contain Key    ${body}    database
    ${db}=    Set Variable    ${body}[database]
    Should Be Equal    ${db}[status]    connected

Health Check Uptime Is Positive
    ${body}=    Health Check
    Should Be True    ${body}[uptime_seconds] >= 0    Uptime should be non-negative

Health Check Response Time Is Acceptable
    ${start}=    Evaluate    time.time()    time
    Health Check
    ${end}=    Evaluate    time.time()    time
    ${elapsed}=    Evaluate    ${end} - ${start}
    Should Be True    ${elapsed} < 2.0    Health check response time should be under 2 seconds

Repeated Health Checks Are Consistent
    ${bodies}=    Create List
    FOR    ${i}    IN RANGE    5
        ${body}=    Health Check
        Append To List    ${bodies}    ${body}[status]
    END
    FOR    ${status}    IN    @{bodies}
        Should Be Equal    ${status}    ok
    END
