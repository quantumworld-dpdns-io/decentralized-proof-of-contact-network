{{-/*
POI Network helpers
Usage: {{ include "poi-network.helpers.name" . }}
*/}}

{{-/*
Expand the name of the chart.
*/}}
{{- define "poi-network.name" -}}
{{- default .Chart.Name .Values.global.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{-/*
Create a default fully qualified app name.
*/}}
{{- define "poi-network.fullname" -}}
{{- if .Values.global.fullnameOverride }}
{{- .Values.global.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.global.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{-/*
Create chart name and version as used by the chart label.
*/}}
{{- define "poi-network.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{-/*
Common labels
*/}}
{{- define "poi-network.labels" -}}
helm.sh/chart: {{ include "poi-network.chart" . }}
{{ include "poi-network.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{-/*
Selector labels
*/}}
{{- define "poi-network.selectorLabels" -}}
app.kubernetes.io/name: {{ include "poi-network.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{-/*
Create the name of the service account to use for a component
Usage: {{ include "poi-network.serviceAccountName" (dict "component" "node" "context" $) }}
*/}}
{{- define "poi-network.serviceAccountName" -}}
{{- $saKey := printf "serviceAccounts.%s" .component }}
{{- $saConfig := dig .component (dict) .context.Values.serviceAccounts }}
{{- if $saConfig.create }}
{{- default (printf "%s-%s" (include "poi-network.fullname" .context) .component) $saConfig.name }}
{{- else }}
{{- "default" }}
{{- end }}
{{- end }}

{{-/*
Image reference
Usage: {{ include "poi-network.image" (dict "image" .Values.node.image "global" .Values.global) }}
*/}}
{{- define "poi-network.image" -}}
{{- $registry := .global.imageRegistry | default "" }}
{{- $repository := .image.repository }}
{{- $tag := .image.tag | default .global.imageTag | default .chartVersion }}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag }}
{{- else }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

{{-/*
Image pull policy
Usage: {{ include "poi-network.imagePullPolicy" (dict "image" .Values.node.image "global" .Values.global) }}
*/}}
{{- define "poi-network.imagePullPolicy" -}}
{{- if .image.pullPolicy }}
{{- .image.pullPolicy }}
{{- else }}
{{- .global.imagePullPolicy | default "IfNotPresent" }}
{{- end }}
{{- end }}

{{-/*
Get the component name from a template context
Usage: {{ include "poi-network.componentName" (dict "component" "node") }}
*/}}
{{- define "poi-network.componentName" -}}
{{- printf "%s-%s" (include "poi-network.fullname" .context) .component }}
{{- end }}

{{-/*
Environment variables helper
Usage: {{ include "poi-network.env" (dict "env" .Values.node.env "context" $) }}
*/}}
{{- define "poi-network.env" -}}
{{- range $key, $value := .env }}
- name: {{ $key }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{-/*
Security context helper
Usage: {{ include "poi-network.securityContext" .Values.node.securityContext }}
*/}}
{{- define "poi-network.securityContext" -}}
allowPrivilegeEscalation: false
capabilities:
  drop:
    - ALL
{{- if .runAsUser }}
runAsUser: {{ .runAsUser }}
{{- end }}
{{- if .runAsGroup }}
runAsGroup: {{ .runAsGroup }}
{{- end }}
{{- if .runAsNonRoot }}
runAsNonRoot: {{ .runAsNonRoot }}
{{- end }}
{{- end }}

{{-/*
Probe helper
Usage: {{ include "poi-network.probe" (dict "type" "liveness" "port" 3000) }}
*/}}
{{- define "poi-network.probe" -}}
httpGet:
  path: /health
  port: {{ .port }}
initialDelaySeconds: {{ .initialDelaySeconds | default 15 }}
periodSeconds: {{ .periodSeconds | default 20 }}
timeoutSeconds: {{ .timeoutSeconds | default 5 }}
failureThreshold: {{ .failureThreshold | default 3 }}
successThreshold: {{ .successThreshold | default 1 }}
{{- end }}

{{-/*
Resource helper
Usage: {{ include "poi-network.resources" .Values.node.resources }}
*/}}
{{- define "poi-network.resources" -}}
requests:
  cpu: {{ .requests.cpu | default "100m" }}
  memory: {{ .requests.memory | default "128Mi" }}
limits:
  cpu: {{ .limits.cpu | default "500m" }}
  memory: {{ .limits.memory | default "512Mi" }}
{{- end }}

{{-/*
Pod anti-affinity helper
*/}}
{{- define "poi-network.podAntiAffinity" -}}
{{- if .required }}
requiredDuringSchedulingIgnoredDuringExecution:
  - labelSelector:
      matchExpressions:
        - key: app.kubernetes.io/component
          operator: In
          values:
            - {{ .component }}
    topologyKey: kubernetes.io/hostname
{{- end }}
{{- if .preferred }}
preferredDuringSchedulingIgnoredDuringExecution:
{{- range .preferred }}
  - weight: {{ .weight }}
    podAffinityTerm:
      labelSelector:
        matchExpressions:
          - key: app.kubernetes.io/component
            operator: In
            values:
              - {{ $.component }}
      topologyKey: {{ .topologyKey }}
{{- end }}
{{- end }}
{{- end }}

{{-/*
Namespace
*/}}
{{- define "poi-network.namespace" -}}
{{- .Values.namespace.name | default "poi-network" }}
{{- end }}

{{-/*
Fully qualified name for a component
*/}}
{{- define "poi-network.componentFQN" -}}
{{- $component := .component }}
{{- printf "%s-%s.%s.svc.cluster.local" (include "poi-network.fullname" .context) $component (include "poi-network.namespace" .context) }}
{{- end }}
