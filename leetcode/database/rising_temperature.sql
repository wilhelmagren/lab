SELECT Today.id FROM Weather AS Today
INNER JOIN Weather AS Yesterday
    ON ((Today.recordDate - INTERVAL '1 DAY') = Yesterday.recordDate)
WHERE Today.temperature > Yesterday.temperature;
