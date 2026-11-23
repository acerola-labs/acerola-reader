package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * One choice in a list (language, order, category), the desktop's radio list of buttons: a 48dp
 * pill row, the selected one in `bg-primary-soft text-primary` with a check at the end.
 */
@Composable
fun Acerola.Component.OptionRow(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    icon: ImageVector? = null,
    description: String? = null,
    dotColor: Color? = null,
) {
    val brand = AcerolaExtendedTheme.colors
    val content = if (selected) MaterialTheme.colorScheme.primary else brand.foreground

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .heightIn(min = 48.dp)
                .clip(AcerolaShapes.Full)
                .background(if (selected) brand.primarySoft else Color.Transparent)
                .semantics {
                    this.selected = selected
                    role = Role.RadioButton
                }.clickable(onClick = onClick)
                .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        if (dotColor != null) {
            // A user-chosen color (bookmark), shown as a dot in front of its name.
            Box(modifier = Modifier.size(12.dp).clip(AcerolaShapes.Full).background(dotColor))
        } else if (icon != null) {
            Icon(imageVector = icon, contentDescription = null, tint = content, modifier = Modifier.size(SizeTokens.IconSmall))
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = label,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = if (selected) FontWeight.Black else FontWeight.Medium),
                color = content,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (description != null) {
                Text(text = description, style = MaterialTheme.typography.bodySmall, color = brand.mutedForeground)
            }
        }
        if (selected) {
            Icon(imageVector = Icons.Default.Check, contentDescription = null, tint = content, modifier = Modifier.size(SizeTokens.IconSmall))
        }
    }
}

/** Header of a sheet: `font-black` title and an optional muted lead, with the sheet's padding. */
@Composable
fun Acerola.Component.SheetHeader(
    title: String,
    modifier: Modifier = Modifier,
    lead: String? = null,
) {
    Column(
        modifier = modifier.fillMaxWidth().padding(horizontal = SpacingTokens.Huge, vertical = SpacingTokens.Small),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall),
    ) {
        Text(text = title, style = MaterialTheme.typography.titleLarge, color = AcerolaExtendedTheme.colors.foreground)
        if (lead != null) {
            Text(text = lead, style = MaterialTheme.typography.bodyMedium, color = AcerolaExtendedTheme.colors.mutedForeground)
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun OptionRowPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.padding(SpacingTokens.Large)) {
            Acerola.Component.SheetHeader(title = "Idioma dos metadados")
            Acerola.Component.OptionRow(label = "Português (Brasil)", selected = true, onClick = {})
            Acerola.Component.OptionRow(label = "English", selected = false, onClick = {})
        }
    }
}
