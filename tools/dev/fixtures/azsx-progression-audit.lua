-- Read-only, explicit diagnostic sampling; no input or gameplay writes.
if not AZSXDiagnosticConfig or not AZSXDiagnosticConfig.progression then return end
AZSXProgressionAudit={frames=0,live=0,betweenSnapshots=0,visible=0,maxStep=0}
local f=CreateFrame("Frame",nil,UIParent)
local oldSample,oldX,oldY,oldAX,oldAY,oldRunX,oldRunY
f:SetScript("OnUpdate",function()
 local a=AZSXProgressionAudit;a.frames=a.frames+1
 if type(AZS_GetPlayerPosition)~="function"then a.api="missing";return end
 local p,err=AZS_GetPlayerPosition();a.api=err and err.code or"ok"
 if type(p)~="table"then return end
 a.live=a.live+1;a.map=p.mapId;a.run=p.runId
 local arrow=getglobal("AZSXUIPlayerMinimapArrow")
 if arrow and arrow:IsVisible()then
  a.visible=a.visible+1
  local _,_,_,x,y=arrow:GetPoint()
  if type(x)=="number"and type(y)=="number"then
   if oldAX then local step=math.sqrt((x-oldAX)^2+(y-oldAY)^2);if step>a.maxStep then a.maxStep=step end end
   oldAX,oldAY=x,y;a.arrowX=x;a.arrowY=y
  end
 end
 if oldSample and p.sampleId~=oldSample and(p.x~=oldX or p.y~=oldY)and AZS_RUN and AZS_RUN.px==oldRunX and AZS_RUN.py==oldRunY then a.betweenSnapshots=a.betweenSnapshots+1 end
 oldSample,oldX,oldY=p.sampleId,p.x,p.y
 oldRunX,oldRunY=AZS_RUN and AZS_RUN.px,AZS_RUN and AZS_RUN.py
 a.level=type(UnitLevel)=="function"and UnitLevel("player")or nil
 a.xp=type(UnitXP)=="function"and UnitXP("player")or nil
 a.phase=AZS_GetPhase and AZS_GetPhase()
end)
