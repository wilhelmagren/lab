SELECT class FROM
(
    SELECT class, COUNT(class) AS cnt FROM Courses
    GROUP BY class
)
WHERE cnt >= 5;
