package tech.skagedal.assistant.tracker

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import java.io.StringWriter
import java.nio.file.FileSystems
import java.time.LocalDate
import java.time.LocalTime

internal class SerializerTest {
    @Test
    internal fun `serialize some simple documents`() {
        val repository = Repository(FileSystems.getDefault())
        val serializer = Serializer()
        assertEquals(
            """
                [monday 2020-07-06]

                [tuesday 2020-07-07]

                [wednesday 2020-07-08]

                [thursday 2020-07-09]

                [friday 2020-07-10]


            """.trimIndent(),
            serializer.documentToString(repository.defaultDocument(LocalDate.of(2020, 7, 11)))
        )
    }

    @Test
    internal fun `test both ways`() {
        val serializer = Serializer()
        val serializedForm = """
            [monday 2020-07-13]
            * Vacation
            # Came back from Jämtland

            [tuesday 2020-07-14]
            * 08:32-12:02
            * 12:30-13:01
            * 13:45-18:03

            [wednesday 2020-07-15]
            * 11:00-18:00

            [thursday 2020-07-16]
            * 08:00-12:00
            * VAB 13:00-17:00

            [friday 2020-07-17]
            * 08:12-
            
        """.trimIndent()
        val document = Document(
            listOf(
                Line.DayHeader(LocalDate.of(2020, 7, 13)),
                Line.SpecialDay("Vacation"),
                Line.Comment("Came back from Jämtland"),
                Line.Blank,
                Line.DayHeader(LocalDate.of(2020, 7, 14)),
                Line.ClosedShift(LocalTime.of(8, 32), LocalTime.of(12, 2)),
                Line.ClosedShift(LocalTime.of(12, 30), LocalTime.of(13, 1)),
                Line.ClosedShift(LocalTime.of(13, 45), LocalTime.of(18, 3)),
                Line.Blank,
                Line.DayHeader(LocalDate.of(2020, 7, 15)),
                Line.ClosedShift(LocalTime.of(11, 0), LocalTime.of(18, 0)),
                Line.Blank,
                Line.DayHeader(LocalDate.of(2020, 7, 16)),
                Line.ClosedShift(LocalTime.of(8, 0), LocalTime.of(12, 0)),
                Line.SpecialShift("VAB", LocalTime.of(13, 0), LocalTime.of(17, 0)),
                Line.Blank,
                Line.DayHeader(LocalDate.of(2020, 7, 17)),
                Line.OpenShift(LocalTime.of(8, 12))
            )
        )
        assertEquals(serializedForm, serializer.documentToString(document))
    }

    fun Serializer.documentToString(document: Document) =
        StringWriter().apply { writeDocument(document, this) }.toString()
}